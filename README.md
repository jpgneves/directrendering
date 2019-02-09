# directrendering

Yet another Rust binding library to `libdrm`, with ergonomics I personally enjoy.

## Example

This example has been tested on FreeBSD 12.0-RELEASE. Note that you MUST give the
resulting executable the right permissions, e.g.:

```bash
$ sudo chown root:wheel target/debug/foo
$ sudo chmod ug+s target/debug/foo
```

```rust
extern crate directrendering;

use directrendering::Device;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};

const CARD0: &str = "/dev/dri/card0";

fn main() -> std::io::Result<()> {
    if directrendering::is_drm_available() {
        println!("DRM is available!");

        let file = OpenOptions::new().read(true).write(true).open(CARD0)?;
        
        let dev = Device::new(&file);
        
        dev.as_master(|m| {
            println!("Became DRM Master!");
        
            let r = m.get_resources()?;
            println!("DRM Mode resources: {:?}", r);
            
            for c in r.connectors.iter() {
                let connector = m.get_connector(*c)?;
                println!("Connector {:?}: {:?}", c, connector);
            };
            
            Ok(())
        })
    } else {
        Err(Error::new(ErrorKind::Other, "DRM is not available"))
    }
}
```
