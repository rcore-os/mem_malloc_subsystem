# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.16

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:


#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:


# Remove some rules from gmake that .SUFFIXES does not remove.
SUFFIXES =

.SUFFIXES: .hpux_make_needs_suffix_list


# Suppress display of executed commands.
$(VERBOSE).SILENT:


# A target that is always out of date.
cmake_force:

.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/bin/cmake

# The command to remove a file.
RM = /usr/bin/cmake -E remove -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench

# Include any dependencies generated for this target.
include CMakeFiles/mstress.dir/depend.make

# Include the progress variables for this target.
include CMakeFiles/mstress.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/mstress.dir/flags.make

CMakeFiles/mstress.dir/mstress/mstress.c.o: CMakeFiles/mstress.dir/flags.make
CMakeFiles/mstress.dir/mstress/mstress.c.o: /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench/mstress/mstress.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/mstress.dir/mstress/mstress.c.o"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/mstress.dir/mstress/mstress.c.o   -c /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench/mstress/mstress.c

CMakeFiles/mstress.dir/mstress/mstress.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/mstress.dir/mstress/mstress.c.i"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench/mstress/mstress.c > CMakeFiles/mstress.dir/mstress/mstress.c.i

CMakeFiles/mstress.dir/mstress/mstress.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/mstress.dir/mstress/mstress.c.s"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench/mstress/mstress.c -o CMakeFiles/mstress.dir/mstress/mstress.c.s

# Object files for target mstress
mstress_OBJECTS = \
"CMakeFiles/mstress.dir/mstress/mstress.c.o"

# External object files for target mstress
mstress_EXTERNAL_OBJECTS =

mstress: CMakeFiles/mstress.dir/mstress/mstress.c.o
mstress: CMakeFiles/mstress.dir/build.make
mstress: CMakeFiles/mstress.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking C executable mstress"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/mstress.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/mstress.dir/build: mstress

.PHONY : CMakeFiles/mstress.dir/build

CMakeFiles/mstress.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/mstress.dir/cmake_clean.cmake
.PHONY : CMakeFiles/mstress.dir/clean

CMakeFiles/mstress.dir/depend:
	cd /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/bench /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench /home/abc/桌面/git/arceos/malloc/testbench/mimalloc-bench/out/bench/CMakeFiles/mstress.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/mstress.dir/depend
