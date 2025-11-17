use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "arith";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            match run_test(&test) {
                TestOutcome::Pass => println!("✅ PASS"),
                TestOutcome::Fail { expected, actual } => {
                    println!("❌ FAIL");
                    println!("\nExpected:\n{}", expected);
                    println!("\nActual:\n{}", actual);

                    // Show byte-by-byte comparison for first difference
                    let exp_bytes: Vec<u8> = expected.bytes().collect();
                    let act_bytes: Vec<u8> = actual.bytes().collect();

                    for (i, (e, a)) in exp_bytes.iter().zip(act_bytes.iter()).enumerate() {
                        if e != a {
                            println!("\nFirst difference at byte {}:", i);
                            println!("  Expected: {:?} (byte {})", *e as char, e);
                            println!("  Actual:   {:?} (byte {})", *a as char, a);
                            break;
                        }
                    }

                    if exp_bytes.len() != act_bytes.len() {
                        println!("\nLength difference:");
                        println!("  Expected: {} bytes", exp_bytes.len());
                        println!("  Actual:   {} bytes", act_bytes.len());
                    }
                }
                TestOutcome::GrammarParseError(e) => println!("❌ Grammar Parse Error: {}", e),
                TestOutcome::InputParseError(e) => println!("❌ Input Parse Error: {}", e),
                TestOutcome::Skip(reason) => println!("⏭️  SKIP: {}", reason),
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
