#!/bin/sh

mkdir drm
wget -O drm/drm.h https://github.com/torvalds/linux/raw/master/include/uapi/drm/drm.h
wget -O drm/drm_mode.h https://github.com/torvalds/linux/raw/master/include/uapi/drm/drm_mode.h
echo "LIBDRM_INCLUDE_PATH=${PWD}/drm" >> $GITHUB_ENV
echo "BINDGEN_EXTRA_CLANG_ARGS=-D __user= ${BINDGEN_EXTRA_CLANG_ARGS}" >> $GITHUB_ENV
