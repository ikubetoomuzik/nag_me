// binary testing
//

use async_std::{process, task};

fn main() {
    task::block_on(async {
        println!("Child process output...");
        process::Command::new("/home/curtis/scripts/test.sh")
            .stdout(process::Stdio::inherit())
            .status()
            .await
            .expect("failed.");
    });
    println!("In progress...");
}
