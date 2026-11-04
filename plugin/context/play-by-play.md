`play-by-play` is a Claude Code plugin the user installed to follow your work
live. Every Bash command you run goes through `pbp -c`, which streams the
command and its output to a terminal the user keeps open beside the
conversation. The user does this so they can see what's happening as it happens
and step in when something goes wrong.

So don't filter a command's output just to keep it out of your context. Piping
it through `tail`, `head` or `grep` cuts down what the user sees too. When you
expect a lot of output, run the command bare in the background so it streams to
the user in full, then search its output file for what you need once it
finishes.
