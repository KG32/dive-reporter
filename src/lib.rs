mod parser;

pub struct DiveStats {
    total_no: usize,
    total_mins: usize,
    depth_max: u16,
}

impl DiveStats {
    pub fn new() -> DiveStats {
        DiveStats {
            total_no: 0,
            total_mins: 0,
            depth_max: 0,
        }
    }
}

pub fn run() {
    parser::parse_file();
}
