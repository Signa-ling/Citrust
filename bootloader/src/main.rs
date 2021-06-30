#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;

use uefi::{
    prelude::*,
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

    writeln!(&mut system_table.stdout(), "Call Kernel file").unwrap();

    // カーネルファイルの呼び出し
    let kernel_file_handle = root_dir.open("\\kernel.elf", FileMode::Read, FileAttribute::READ_ONLY).unwrap_success();
    let mut kernel_file = match kernel_file_handle.into_type().unwrap_success() {
        FileType::Regular(file) => file,
        _ => panic!("kernel file is not regular file"),
    };
   
    // 必要バッファ数の確認: required size = 102
    let check_file_size = kernel_file.get_info::<FileInfo>(&mut []).expect_error("");
    writeln!(&mut system_table.stdout(), "required size = {:?}", check_file_size).unwrap();

    loop {}
    //Status::SUCCESS
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
