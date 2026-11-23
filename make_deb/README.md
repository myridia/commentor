## Create Debian Package Structure
```
dh_make -i --createorig -e info@myridia.com -c gpl3 -y
```

## Build the Package
```
cd src
dpkg-buildpackage -us -uc -b
```

## Install the deb package into the os with apt, it will install all depencies as well
```
apt install -f ./commentor_0.1.0-1_amd64.deb
```

## Or install the deb package into the os with dpkg
```
sudo dpkg -i commentor_0.1.0-1_amd64.deb
```

## Review the installed files of the installed deb package
```
dpkg -L commentor
```

## Remove the deb package from the os
```
sudo apt-get remove commentor --purge
```
