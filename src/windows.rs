use std::io::Result;

use windows_registry::CURRENT_USER;

use crate::ID;

mod windows_single;

pub fn register(scheme: &str) -> Result<()> {
    let key_base = format!("Software\\Classes\\{}", scheme);

    let exe = dunce::simplified(&crate::current_exe()?)
        .display()
        .to_string();

    let key_reg = CURRENT_USER.create(&key_base)?;
    key_reg.set_string(
        "",
        &format!(
            "URL:{} protocol",
            ID.get().expect("register() called before prepare()")
        ),
    )?;
    key_reg.set_string("URL Protocol", "")?;

    let icon_reg = CURRENT_USER.create(format!("{key_base}\\DefaultIcon"))?;
    icon_reg.set_string("", &format!("{exe},0"))?;

    let cmd_reg = CURRENT_USER.create(format!("{key_base}\\shell\\open\\command"))?;

    cmd_reg.set_string("", &format!("\"{exe}\" \"%1\""))?;

    Ok(())
}

pub fn unregister(scheme: &str) -> Result<()> {
    CURRENT_USER.remove_tree(format!("Software\\Classes\\{}", scheme))?;

    Ok(())
}

pub fn listen<F: FnMut(String) + Sync + Send + 'static>(mut handler: F) -> Result<()> {
    windows_single::init(Box::new(move |args, _| {
        handler(args.join(" "));
    }));

    Ok(())
}

pub fn prepare(identifier: &str) {
    ID.set(identifier.to_string())
        .expect("prepare() called more than once with different identifiers.");
}
