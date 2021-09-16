use std::env;
use std::process::Command;
use std::string::FromUtf8Error;

struct Error(String);

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(err.to_string())
    }
}

fn get_args() -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();
    for arg in env::args() {
        vec.push(arg);
    }
    vec
}

fn shell<S: Into<String>>(command: S, throw: bool) -> Result<String, Error> {
    let output = Command::new("bash").arg("-c").arg(command.into())
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    if !stderr.is_empty() && throw {
        return Err(Error(stderr));
    }
    let stdout = String::from_utf8(output.stdout)?;
    Ok(format!("{}\n{}", stdout, stderr))
}

fn execute() -> Result<(), Error> {
    let args = get_args();
    if args.len() < 3 {
        return Err(Error(String::from("Simple AES-256-CBC encrypted transfer.sh software\nDependencies: openssh, curl\nUSAGE: transfer [FILE/URL] [PASSWORD]")));
    }
    let file = args.get(1).ok_or(Error(String::from("No FILE/URL argument")))?;
    let pass = args.get(2).ok_or(Error(String::from("No password argument")))?;
    if file.starts_with("https://transfer.sh") {
        let i = file.rfind('/').unwrap() + 1;
        let name = String::from(&file[i..]);
        println!("Downloading {}", name);
        shell(format!("curl {link} -o {file}", link = file, file = "/tmp/transfer_download.enc"), false)?;
        println!("Decrypting");
        shell(
            format!("openssl aes-256-cbc -d -pbkdf2 -iter 10000 -a -salt -in /tmp/transfer_download.enc -out {file} -pass file:<( echo -n \"{pass}\" )",
                    file = name,
                    pass = pass
            ), true)?;
        shell("rm -f /tmp/transfer_download.enc", true)?;
    } else {
        let mut name = String::new();
        if file.contains('/') {
            let i = file.rfind('/').unwrap() + 1;
            name.push_str(&file[i..]);
        } else {
            name.push_str(file.as_str());
        }
        println!("Encrypting {}", name);
        shell(
            format!("openssl aes-256-cbc -pbkdf2 -iter 10000 -a -salt -in {file} -out /tmp/transfer.enc -pass file:<( echo -n \"{pass}\" )",
                    file = file,
                    pass = pass
            ), true)?;
        println!("Uploading");
        let link = shell(format!("curl -T {file} https://transfer.sh/{name}", file = "/tmp/transfer.enc", name = name), false)?;
        println!("Link: {}", link);
        shell("rm -f /tmp/transfer.enc", true)?;
    }
    Ok(())
}

fn main() {
    match execute() {
        Ok(_) => {
            println!("OK");
        }
        Err(err) => {
            eprintln!("{}", err.0);
        }
    }
}
