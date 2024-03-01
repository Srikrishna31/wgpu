use crate::{model, texture};
use cfg_if::cfg_if;
/// By design, you can't access files on a user's filesystem in Web Assembly. Instead, we'll serve
/// those files up using a web serve and then load those files into our code using an http request.
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = {
        let origin = location.origin().unwrap();
        if !origin.ends_with("learn-wgpu") {
            format!("{}/learn-wgpu", origin)
        } else {
            origin
        }
    };
    let base = reqwest::Url::parse(&format!("{origin}/")).unwrap();
    base.join(file_name).unwrap()
}

pub(crate) async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url).await?.text().await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("models").join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub(crate) async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let bytes = reqwest::get(url).await?.bytes().await?.to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR")).join("models").join(file_name);
            let bytes = std::fs::read(path)?;
        }
    }

    Ok(bytes)
}
