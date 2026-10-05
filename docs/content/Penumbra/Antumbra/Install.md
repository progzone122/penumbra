Installing Antumbra is a process that might vary depending on your operating system.

### Windows

On Windows, you can download the latest release of Antumbra from the [GitHub releases page](https://github.com/shomykohai/penumbra/releases).
You'll find a file named `antumbra.exe` in the assets section of the release. Download it and place it in a directory of your choice.

You'll now need to set up drivers.
If you don't plan on using the `Linecode` exploit, you can use stock MediaTek USB drivers, which you can find with a web search.

If you do plan on using the `Linecode` exploit, you'll need to install either WinUSB or LibUSB drivers.
You can use [Zadig](https://zadig.akeo.ie/) to install the drivers, or the provided driver installer in the releases.
Generally, you'd want to replace the driver, but for LibUSB, you can install it as a filter driver, unless no device is detected.

The provided driver installer will replace the driver.

> [!NOTE]
> UsbDk is not supported by Penumbra, because of stability issues and the fact it's unmaintained.
> If already installed, it might not cause issues, but be aware of it.

### Linux based distributions

On Linux, you can download the latest release of Antumbra from the [GitHub releases page](https://github.com/shomykohai/penumbra/releases).

If you plan to use `libusb` backend, you might need to install `libusb` dependency.

After installing the dependencies, you'll need to set up udev rules and group access for your user to access USB devices.

You can get the udev rules from [mtkclient](https://github.com/bkerler/mtkclient/blob/main/Setup/Linux/52-mtk.rules)

If the udev rules are not set up correctly, you might get errors like "No device found" or "Permission denied", in which case you should check your setup again, or run Antumbra as root (not recommended).

```sh
$ ./antumbra help
```

### Arch Linux

On Arch Linux, you can install Antumbra with the provided PKGBUILD in the repo.

```sh
$ git clone https://github.com/shomykohai/penumbra.git
$ cd penumbra/app/res/arch
$ makepkg -si
```

### NixOS

On NixOS, you can install Antumbra either with the `flake.nix` provided in the repo, or using [frostix](https://github.com/shomykohai/frostix).


With penumbra flake

```nix
{
  inputs = {
    penumbra.url = "github:shomykohai/penumbra";
  };

  outputs = { self, penumbra }: {
    packages.x86_64-linux.antumbra = penumbra.packages.x86_64-linux.antumbra;
  };
}
```

With frostix

```nix
## flake.nix
{
  inputs = {
    frostix.url = "github:shomykohai/frostix";
  };
}

## configuration.nix
{
  environment.systemPackages = with pkgs; [
    frostix.packages.x86_64-linux.antumbra
  ];
}
```

### Android / Termux

> [!NOTE]
> Currently, you'll need Root access to use Antumbra on Android, as it requires access to USB devices.
> If you find a way to use it without root, open a discussion on the repo!!

On Android, you can install Antumbra using Termux.

```sh
# Install Tur repo for accessing rust-nightly
$ pkg install tur-repo 
$ pkg install rust
$ pkg install rustc-nightly cargo
$ source $PREFIX/etc/profile.d/rust-nightly.sh
$ git clone https://github.com/shomykohai/penumbra.git
$ cd penumbra
$ cargo build --release
$ su
# ./target/release/antumbra
```

> [!NOTE]
> On Android, only libusb backend is supported.
