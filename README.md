# scan

`scan` extracts substrings and builds output using pattern or delimiter.


```shell
# extract time from access log
$ cat access.log | scan -p 'time:(.+?)\s+' {1}
2020-11-29T00:03:10+09:00
2020-11-29T00:11:01+09:00
2020-11-29T00:11:02+09:00
2020-11-29T00:15:55+09:00
2020-11-29T00:21:03+09:00
2020-11-29T00:21:19+09:00
2020-11-29T00:21:58+09:00

# extract 2nd column from csv file
$ cat hoge.csv | scan -d , {2}
hogehoge
fugafuga
```

## setup

Download binary from [Release page](https://github.com/genya0407/scan-rust/releases) and extract to `$PATH` dir.

## examples

### pattern

You can use regular expressions supported by [Regex](https://docs.rs/regex/1.4.2/regex/#syntax).

```shell
$ cat regex.txt
hogehoge_nyan
hohho_nyan

# using pattern
$ cat regex.txt | scan -p "(.+?)_(.+)" {1},{2}
hogehoge,nyan
hohho,nyan

# using pattern with named capture
$ cat regex.txt | scan -p "(?P<left>.+?)_(?P<right>.+)" {left}:{right}
hogehoge:nyan
hohho:nyan
```

### delimiter

Delimiter is also regular expression.

```shell
$ cat hoge.csv
aaa,bbb,ccc
xxx,yyy,zzz

$ cat hoge.csv | scan -d , {3}
ccc
zzz

# default delimiter is '\s+'
$ cat hoge.tsv
aaa     bbb     ccc
xxx     yyy     zzz

$ cat hoge.tsv | scan {2}
bbb
yyy
```

