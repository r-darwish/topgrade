echo This will place the man page in its folder

cd /usr/local/man/
mkdir man8

cd -

sudo cp topgrade.8.gz /usr/local/man/man8/

echo Boom, the man topgrade command now works on your current machine. Thanks for downloading this package.

echo Adding the TopgradeTime command

chmod 777 TimeTopgrade

export PATH=$PATH:.

echo Thanks for downloading this package.
