
echo This will place the man page in its folder

gzip topgrade.8

cd /usr/local/man/
mkdir man8

cd -

sudo cp topgrade.8.gz /usr/local/man/man8/

echo Boom, the man topgrade command now works on your current machine. Thanks.
