# PolyXML AUR (Arch User Repository) Package

This directory contains the packaging files for the **Arch User Repository (AUR)**.

---

## Files

- `PKGBUILD`: Arch Linux package build script.
- `.SRCINFO`: Machine-readable metadata for AUR.

---

## How to Publish / Maintain on the AUR

The AUR is a Git-based repository hosted by Arch Linux.

### Initial Submission (One-time):

1. **Register** an account on [aur.archlinux.org](https://aur.archlinux.org/register).
2. **Add your SSH Public Key** in your AUR account profile (`~/.ssh/id_ed25519.pub` or `~/.ssh/id_rsa.pub`).
3. **Clone the empty AUR repository**:
   ```bash
   git clone ssh://aur@aur.archlinux.org/polyxml.git
   cd polyxml
   ```
4. **Copy the packaging files**:
   ```bash
   cp /path/to/PolyXML/packaging/aur/PKGBUILD .
   cp /path/to/PolyXML/packaging/aur/.SRCINFO .
   ```
5. **Commit and push**:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "feat: initial release 0.1.0"
   git push origin master
   ```

Once pushed, the package is immediately published on the AUR!

---

## How Users Install PolyXML via AUR

Any Arch Linux, Manjaro, or EndeavourOS user can install it via their AUR helper:

```bash
yay -S polyxml
# or
paru -S polyxml
```
