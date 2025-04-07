set max-value-size unlimited
set print repeats unlimited
set python print-stack full
file esp/KERNEL.ELF
gef-remote localhost 12345
tmux-setup
break pkg/kernel/src/interrupt/exceptions.rs:32
break pkg/kernel/src/interrupt/exceptions.rs:42
break pkg/kernel/src/interrupt/exceptions.rs:54