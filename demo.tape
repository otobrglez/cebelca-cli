# VHS tape for the ceb CLI demo.
# Render with:  vhs demo.tape   (produces demo.gif)
# See https://github.com/charmbracelet/vhs for the syntax.

Output demo.gif

Set Shell "zsh"
Set FontSize 16
Set Width 1200
Set Height 700
Set Padding 20
Set TypingSpeed 60ms

# Give the shell a moment to settle.
Sleep 1s

# List partners
Type "ceb partners list"
Enter
Sleep 2s

# Show only paid invoices
Type "ceb invoices list --filter paid"
Enter
Sleep 2s

# Duplicate an existing invoice into a new draft
Type "ceb invoices duplicate 1"
Enter
Sleep 2s

# Finalize (issue) a draft invoice
Type "ceb invoices finalize 66"
Enter
Sleep 2s

# Show invoices for a specific partner (id 7)
Type "ceb partners invoices 7"
Enter
Sleep 3s
