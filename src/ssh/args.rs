/// SSH flags that consume the next argument as their value.
const FLAGS_WITH_VALUE: &[&str] = &[
    "-b", "-c", "-D", "-E", "-e", "-F", "-I", "-i", "-J", "-L", "-l", "-m", "-O", "-o", "-p",
    "-Q", "-R", "-S", "-W", "-w",
];

#[derive(Debug, Clone)]
pub struct SshArgs {
    pub user: Option<String>,
    pub host: String,
    pub port: u16,
    pub passthrough: Vec<String>,
}

impl SshArgs {
    pub fn parse(args: &[String]) -> Self {
        let passthrough: Vec<String> = args.to_vec();

        let mut port: u16 = 22;
        let mut destination: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            if FLAGS_WITH_VALUE.contains(&arg.as_str()) {
                // Flag that takes a value — consume both
                if arg == "-p" {
                    if let Some(val) = args.get(i + 1) {
                        if let Ok(p) = val.parse::<u16>() {
                            port = p;
                        }
                    }
                }
                i += 2; // skip flag and its value
            } else if arg.starts_with('-') {
                // Flag without a value (e.g. -v, -N, -T, etc.)
                i += 1;
            } else {
                // First non-flag argument is the destination
                if destination.is_none() {
                    destination = Some(arg.clone());
                }
                i += 1;
            }
        }

        let dest = destination.unwrap_or_default();
        let (user, host) = if let Some(at) = dest.find('@') {
            let u = dest[..at].to_string();
            let h = dest[at + 1..].to_string();
            (Some(u), h)
        } else {
            (None, dest)
        };

        SshArgs {
            user,
            host,
            port,
            passthrough,
        }
    }
}
