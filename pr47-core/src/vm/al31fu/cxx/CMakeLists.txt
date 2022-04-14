# Note: this file only serves for hint. It is not used by the build system.

cmake_minimum_required(VERSION 2.9)
project(pseudo_project)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

include_directories(include)

add_library(pr47_al31fu_pseudo src/insc.cc src/pseudo.cc)
