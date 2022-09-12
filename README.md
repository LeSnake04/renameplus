# Renameplus

Tool to rename files with smart extra features.

```sh
autorename
LeSnake <dev.lesnake@posteo.de>
Tool to rename files

USAGE:
    renameplus [OPTIONS] <FILE>...

ARGS:
    <FILE>...    File(s)  to be renamed

OPTIONS:
    -c, --copy                   Copy files instead of moving them
    -d, --dry                    Dont perfrom the operations
    -f, --fragile                Crash as soon as a error occurs
    -h, --help                   Print help information
        --loglevel <loglevel>    Set the loglevel [default: WARN] [possible values: OFF, ERROR,
                                 WARN, INFO, DEBUG, TRACE]
    -r, --dirs                   Allow renaming of directories

SIMPLE:
    -p, --prefix <PREFIX>    Prefix to be added to the file
    -s, --suffix <SUFFIX>    Attach text to files
```

## Examples
### Add prefix
```sh
renameplus text.txt video.mp4 -p "my_"
```
renames text.txt to my_test.txt and video.mp4 to my_video.mp4.