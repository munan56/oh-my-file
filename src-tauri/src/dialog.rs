use std::path::PathBuf;

pub fn pick_save_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择接收文件保存目录")
        .pick_folder()
}
