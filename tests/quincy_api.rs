use quincy_tun::{AsyncDevice, ToIpv4Address, ToIpv4Netmask};
use std::io;

async fn async_io_contract(device: &AsyncDevice) -> io::Result<()> {
    let _name = device.name()?;
    device.enabled(true)?;

    let mut packet = [0u8; 1500];
    let len = device.recv(&mut packet).await?;
    device.send(&packet[..len]).await?;
    Ok(())
}

fn address_contract() -> io::Result<()> {
    let _address = "10.0.0.1".ipv4()?;
    let _prefix = "255.255.255.0".prefix()?;
    Ok(())
}

#[cfg(any(
    all(target_os = "linux", not(target_env = "ohos")),
    target_os = "macos",
    target_os = "freebsd",
    target_os = "windows",
))]
fn desktop_builder_contract() -> io::Result<AsyncDevice> {
    let builder = quincy_tun::DeviceBuilder::new()
        .name("tun0")
        .mtu(1400)
        .enable(true)
        .ipv4("10.0.0.1", 24, None);

    #[cfg(unix)]
    let builder = builder.packet_information(false);

    builder.build_async()
}

#[cfg(unix)]
fn unix_fd_ownership_contract(fd: std::os::fd::OwnedFd) -> io::Result<AsyncDevice> {
    use std::os::fd::IntoRawFd;

    // SAFETY: the function contract requires an owned TUN descriptor.
    unsafe { AsyncDevice::from_fd(fd.into_raw_fd()) }
}

#[cfg(target_os = "linux")]
async fn linux_offload_contract(device: &AsyncDevice) -> io::Result<()> {
    use bytes::BytesMut;
    use quincy_tun::{GROTable, IDEAL_BATCH_SIZE, VIRTIO_NET_HDR_LEN};

    let mut original = vec![0u8; VIRTIO_NET_HDR_LEN + u16::MAX as usize];
    let mut receive_buffers = vec![vec![0u8; 1500]; IDEAL_BATCH_SIZE];
    let mut sizes = vec![0; IDEAL_BATCH_SIZE];
    device
        .recv_multiple(&mut original, &mut receive_buffers, &mut sizes, 0)
        .await?;

    let mut table = GROTable::default();
    let mut send_buffers = vec![BytesMut::from(&[0u8; VIRTIO_NET_HDR_LEN + 40][..])];
    device
        .send_multiple(&mut table, &mut send_buffers, VIRTIO_NET_HDR_LEN)
        .await?;
    Ok(())
}

#[test]
fn quincy_public_api_compiles() {
    let _ = async_io_contract;
    let _ = address_contract;

    #[cfg(any(
        all(target_os = "linux", not(target_env = "ohos")),
        target_os = "macos",
        target_os = "freebsd",
        target_os = "windows",
    ))]
    let _ = desktop_builder_contract;

    #[cfg(unix)]
    let _ = unix_fd_ownership_contract;

    #[cfg(target_os = "linux")]
    let _ = linux_offload_contract;
}
