use engine::Printer;

pub struct StandardPrinter;

impl Printer for StandardPrinter {
    fn print(&self, s: &str) {
        println!("{s}");
    }
}
