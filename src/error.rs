use std::process;


pub fn error_at(filen: &String, linen: &i32, charn: &i32, reason: &String) -> ! {
    eprintln!(
        "\x1b[31;1m[ERROR]\x1b[0m: At {}:{}:{}:\n\t{}",
        filen, linen, charn, reason
    );

    process::exit(1);
}
