use std::time::Instant;
use rustixml::runtime_parser::unicode_category_to_rangeset;

fn main() {
    let categories = vec!["L", "Ll", "Lu", "N", "Nd", "P", "C"];
    
    for cat in categories {
        let start = Instant::now();
        let rangeset = unicode_category_to_rangeset(cat);
        let elapsed = start.elapsed();
        
        if let Some(rs) = rangeset {
            println!("Category {}: {:?} ({} ranges)", cat, elapsed, rs.num_ranges());
        } else {
            println!("Category {}: Not recognized", cat);
        }
    }
}
