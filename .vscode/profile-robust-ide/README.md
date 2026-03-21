# Robust IDE Profile (Reusable)

This profile package captures:

- VS Code user settings (`settings.json`)
- Installed extension IDs (`extensions.txt`)
- Script to install all listed extensions (`install_extensions.sh`)

## Import on another machine

1. Copy the `profile-robust-ide` folder to the target machine.
2. Install extensions:
   ```bash
   ./install_extensions.sh
   ```
3. Merge settings from `settings.json` into your VS Code user settings.

Linux Flatpak user settings path:

- `~/.var/app/com.visualstudio.code/config/Code/User/settings.json`

Linux standard user settings path:

- `~/.config/Code/User/settings.json`

## Optional: VS Code built-in Profile export

You can also create a native VS Code profile from this setup:

1. Open Command Palette.
2. Run `Profiles: Create Profile...`.
3. Include Settings + Extensions.
4. Run `Profiles: Export Profile...` to generate a shareable file.
