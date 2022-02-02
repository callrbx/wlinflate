## Wordlist Inflator
Wordlist Inflator (wlinflate) is a simple utility to take an existing wordlist, and quickly customize it for a specific need. 

This tools allows the quick adding of prepended, appended, substituted, and added extensions to a wordlist.

Example usage:
```
13427825 rockyou.txt
❯ time wlinflate -w rockyou.txt -a "prod" -p "acme" -x ".bak" -o inflated_rockyou.txt
wlinflate -w rockyou.txt -a "prod" -p "acme" -x ".bak" -o   1.88s user 0.42s system 99% cpu 2.315 total
❯ wc -l inflated_rockyou.txt
20944368 inflated_rockyou.txt
❯ head inflated_rockyou.txt
0
acme0
0prod
acme0prod
0.bak
acme0.bak
0prod.bak
acme0prod.bak
00
acme00
```

If you find you are running into common naming schemes on an engagement, there is a handy substitution option:
```
❯ cat test_swap.txt
{SWAP}-panel
{SWAP}-manage
❯ wlinflate -w test_swap.txt  -s "acme,ecorp"
acme-panel
ecorp-panel
acme-manage
ecorp-manage
```
This allows you to create targeted but reusable dictionaries.

Help text:
```
wlinflate 0.1.0
icon
simple tool to expand a wordlist with prepends, appends, extensions, and substitutions

USAGE:
    wlinflate [OPTIONS] --wordlist <wordlist>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --append <append>            append wordlist words (csv)
    -x, --extensions <extensions>    extensions to search (csv)
    -o, --output <outfile>           output file
    -p, --prepend <prepend>          prepend wordlist words (csv)
    -s, --swap <swap>                swap in for entries that contain {SWAP} (csv)
    -w, --wordlist <wordlist>        path to wordlist
```