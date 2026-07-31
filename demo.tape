# VHS tape for dns-switcher — https://github.com/charmbracelet/vhs
#
# Run this where dns-switcher is installed and the system dependencies
# (systemd-resolved, iwd, polkit) are available:
#
#     vhs demo.tape
#
# It produces demo.gif next to this file. Commit that gif to the repo root.
#
# Deliberately missing: the Enter that applies a provider. Applying rewrites the
# machine's real DNS configuration and asks polkit for elevation, neither of
# which belongs in an unattended recording. Add it back if you record on a box
# where you don't mind the change — and press r afterwards to reset.
#
# One thing to check before committing the result: the header prints the DNS the
# system is using right now, so recording at home puts your router's address on
# a public repo. redact-dns-gif.py in the cv folder paints over just that.

Output demo.gif
Require dns-switcher

Set Shell "bash"
Set FontSize 16
Set Width 1100
Set Height 430
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

Hide
Type "clear"
Enter
Show

Type "dns-switcher"
Sleep 500ms
Enter
Sleep 2500ms

# Walk down the list — the details pane follows the selection
Down 1
Sleep 900ms
Down 1
Sleep 900ms
Down 1
Sleep 1200ms

# Measure every resolver, so the choice is made on numbers
Type "t"
Sleep 4s

# Back up to the fastest one
Up 2
Sleep 1200ms
Up 1
Sleep 1500ms

Type "q"
Sleep 1s
