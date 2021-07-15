#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
use core::slice::from_raw_parts_mut;

use library::{PixelFormat, FrameBufferConfig};

use byteorder::{ByteOrder, LittleEndian};
use elf_rs::*;
use uefi::{
    prelude::*,
    proto::console::gop::GraphicsOutput,
    proto::loaded_image::LoadedImage,
    proto::media::file::{File, RegularFile, Directory, FileInfo, FileMode, FileType, FileAttribute},
    proto::media::fs::SimpleFileSystem,
    table::boot::{AllocateType, MemoryType},
};


#[entry]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    writeln!(&mut system_table.stdout(), "Hello, world!").unwrap();

    let boot_services = system_table.boot_services();

    // メモリマップの取得
    let memory_map_buffer: &mut [u8] = &mut [0; 4096*4];
    let (_memory_map_key, memory_descriptor_iter) = boot_services.memory_map(memory_map_buffer).unwrap_success();
    
    // ルートディレクトリを開く
    let mut root_dir = unsafe{ open_root_dir(&boot_services, handle).unwrap_success() };
    
    // メモリマップの保存
    {
        struct RegulerFileWriter(RegularFile);
        impl core::fmt::Write for RegulerFileWriter {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                self.0
                    .write(s.as_bytes())
                    .map_err(|_| core::fmt::Error)?
                    .unwrap();
                Ok(())
            }
        }

        // 保存するファイルの作成とFileHandleの取得
        let memory_map_file_handle = root_dir.open("\\memmap", FileMode::CreateReadWrite, FileAttribute::empty()).unwrap_success();

        // memory_map_file_handleのFileTypeがRegularならRegulerFileWriterへ変換
        let mut memory_map_file = match memory_map_file_handle.into_type().unwrap_success() {
            FileType::Regular(file) => RegulerFileWriter(file),
            _ => panic!("\\memmap is a directory"),
        };

        // ヘッダの書き込み
        writeln!(
            memory_map_file,
            "Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute"
        )
        .unwrap();

        // ディスクリプタの書き込み
        for (index, descriptor) in memory_descriptor_iter.enumerate() {
            writeln!(
                memory_map_file,
                "{}, {:x}, {:?}, {:08x}, {}, {:x}",
                index, descriptor.ty.0, descriptor.ty, descriptor.phys_start, descriptor.page_count, descriptor.att
            )
            .unwrap();
        }
    }

    // GOPの取得と画面描画の準備
    let gop = boot_services.locate_protocol::<GraphicsOutput>().unwrap_success();
    let gop = unsafe { &mut *gop.get() };
    let mut fb = gop.frame_buffer();
    let mut fb_addr = fb.as_mut_ptr();
    let fb_size = fb.size();

    // カーネルファイルの呼び出し
    let kernel_file_handle = root_dir.open("\\kernel.elf", FileMode::Read, FileAttribute::READ_ONLY).unwrap_success();
    let mut kernel_file = match kernel_file_handle.into_type().unwrap_success() {
        FileType::Regular(file) => file,
        _ => panic!("kernel file is not regular file"),
    };
   
    // 必要バッファ数の確認: required size = 102
    // let check_file_size = kernel_file.get_info::<FileInfo>(&mut []).expect_error("");
    // writeln!(&mut system_table.stdout(), "required size = {:?}", check_file_size).unwrap();

    // ファイルサイズの取得
    const REQ_BUF: usize = 102;
    let mut buf = [0u8; REQ_BUF];
    let kernel_file_info: &mut FileInfo = kernel_file.get_info::<FileInfo>(&mut buf[..]).unwrap_success();
    let file_size = kernel_file_info.file_size() as usize;

    // メモリの確保
    let kernel_tmp = boot_services.allocate_pool(MemoryType::LOADER_DATA, file_size).unwrap_success();
    let mut kernel_file_buffer = unsafe { from_raw_parts_mut(kernel_tmp as *mut u8, file_size) };
    kernel_file.read(&mut kernel_file_buffer).unwrap_success();
    kernel_file.close();

    // kernel sizeの取得
    let elf = Elf::from_bytes(&kernel_file_buffer).unwrap();
    let mut kernel_start = u64::max_value();
    let mut kernel_end = u64::min_value();
    if let Elf::Elf64(ref e) = elf {
        for program_header in e.program_header_iter() {
            let header = program_header.ph;
            if matches!(header.ph_type(), ProgramType::LOAD) {
                let vaddr = header.vaddr();
                let len = header.memsz();
                kernel_start = core::cmp::min(kernel_start, vaddr);
                kernel_end = core::cmp::max(kernel_end, vaddr + len);
            }
        }
    }

    let load_len = kernel_end - kernel_start;

    // カーネルファイルの読み込み
    let n_pages = (load_len as usize + 0xfff) / 0x1000;

    let page_addr = boot_services
        .allocate_pages(
            AllocateType::Address(kernel_start as usize),
            MemoryType::LOADER_DATA,
            n_pages
        ).unwrap_success();

    if let Elf::Elf64(ref e) = elf {
        for program_header in e.program_header_iter() {
            let header = program_header.ph;
            if matches!(header.ph_type(), ProgramType::LOAD) {
                let segment = program_header.segment();
                let vaddr = header.vaddr();
                let file_len = header.filesz();
                let dst = unsafe { from_raw_parts_mut(vaddr as *mut u8, file_len as usize) };
                for i in 0..file_len as usize {
                    dst[i] = segment[i];
                }
            }
        }
    }

    // エントリポイント用のアドレス
    let ep_buf = unsafe { from_raw_parts_mut((kernel_tmp as u64 + 24) as *mut u8, 8) };
    let kernel_main_addr = LittleEndian::read_u64(&ep_buf);
    writeln!(&mut system_table.stdout(), "main addr = {:x}", kernel_main_addr).unwrap();
    writeln!(&mut system_table.stdout(), "addr: {:?}, size: {:x}", fb_addr, fb_size).unwrap();

    // ブートサービスの停止
    let (_runtime, _desc_itr) = system_table
        .exit_boot_services(handle, &mut memory_map_buffer[..])
        .unwrap_success();
    
    // kernelへの受け渡し
    let kernel_entry = unsafe {
        let f: extern "efiapi" fn(*mut u8, usize) -> ! = core::mem::transmute(kernel_main_addr);
        f
    };
    
    kernel_entry(fb_addr, fb_size);

    loop {}
}

unsafe fn open_root_dir(boot_services: &BootServices, handle: Handle) -> uefi::Result<Directory> {
    let loaded_image = boot_services.handle_protocol::<LoadedImage>(handle)?.unwrap().get();
    let device = (&*loaded_image).device();
    let file_system = boot_services.handle_protocol::<SimpleFileSystem>(device)?.unwrap().get();
    (&mut *file_system).open_volume()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
