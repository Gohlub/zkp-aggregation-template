program in /lib is the one being proved
/program is the aggregator
/script is where you write to the zkvm


from within fibonacci/program run: `cargo prove build --elf-name fibonacci-elf` -- computation we
want to prove
from within program run: `cargo prove build --elf-name aggregator-elf` -- the actual
aggregator program, both will be needed for the script


this repo right now is set up only to declare a root, not update it