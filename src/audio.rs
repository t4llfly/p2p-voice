use cpal::traits::{DeviceTrait, HostTrait};

pub fn get_audio_devices() -> (Vec<String>, Vec<String>) {
    let host = cpal::default_host();

    let inputs = host
        .input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let outputs = host
        .output_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    (inputs, outputs)
}

pub fn find_device_by_name(host: &cpal::Host, name: &str, is_input: bool) -> Option<cpal::Device> {
    let devices = if is_input {
        host.input_devices().ok()?
    } else {
        host.output_devices().ok()?
    };

    for device in devices {
        if let Ok(dev_name) = device.name() {
            if dev_name == name {
                return Some(device);
            }
        }
    }
    None
}

pub fn err_fn(err: cpal::StreamError) {
    eprintln!("Ошибка аудио: {}", err);
}
