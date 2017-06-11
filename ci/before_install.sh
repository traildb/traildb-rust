export LLVM_VERSION_TRIPLE="3.9.0"
export LLVM=clang+llvm-${LLVM_VERSION_TRIPLE}-x86_64-linux-gnu-ubuntu-14.04

wget http://llvm.org/releases/${LLVM_VERSION_TRIPLE}/${LLVM}.tar.xz
mkdir llvm
tar -xf ${LLVM}.tar.xz -C llvm --strip-components=1

export LLVM_CONFIG_PATH=`pwd`/llvm/bin/llvm-config


wget https://github.com/traildb/traildb/archive/0.6.tar.gz
tar -zxf 0.6.tar.gz
pushd traildb-0.6
./waf configure
./waf build
./waf install
