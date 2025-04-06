# Akama
A xmpp client written in rust with iced-rs. (As Of Now this app is no where near an actuall usable client)
### Warning:
 This is a learning project for me, I want to learn and experiment both with xmpp protocol and iced. This means that I don't know what the hell I am doing. I will add new features and improve this as I learn
#


![Screenshot From 2025-04-06 23-47-23](https://github.com/user-attachments/assets/7c47ac23-8e09-43e6-a8a9-5c6c35913944)

## Build
### Nixos
If you're on Nixos or using nix-shell, then run the following (I will add a flake in the future)
```
git clone https://github.com/qecu/akama.git
cd akama
nix-shell -p libxkbcommon libGL wayland
cargo run
```
### Other Distros
make sure these libs are installed on your system `libxkbcommon` `libGL` `wayland`, then run the following
```
git clone https://github.com/qecu/akama.git
cd akama
cargo run
```
### Windows and Macos
```
git clone https://github.com/qecu/akama.git
cd akama
cargo run
```
Now hope and pray nothing goes wrong. (I will add concrete support in the future)
