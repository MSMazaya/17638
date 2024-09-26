# Specify the compiler to use
CC=arm-none-eabi-gcc
# Specify the assembler to use
AS=arm-none-eabi-as
# Specity the linker to use
LD=arm-none-eabi-ld
# Specify the name of the .elf output file
FIRMWARE=main.elf
# Specify the name of the linker script
LINKERFILE=linker_script.ld
# Specify the name of the Map file
MAPFILE=main.map
# Compiler flags
CCFLAGS=-mcpu=cortex-m4 -mthumb -g 

# Set up where the ARM toolchain is installed.  Currently it's set to nothing
#    and has to be pointed to the directory where the toolchain is installed.
# TODO: Set up the path to your toolchain
#TOOLCHAIN=


# Tell the linker where to find the libraries -> important: use thumb versions
# We want to target the v7-m thumb library for this project.
LIBSPEC=-L $(TOOLCHAIN)/arm-none-eabi/lib/thumb/v7-m

# List the object files involved in this project
OBJS=	init.o main.o 

# The default 'target' (output) is main.elf and it depends on the object files being there.
# These object files are linked together to create main.elf
elf : $(OBJS)
	$(LD) $(OBJS) $(LIBSPEC) -T $(LINKERFILE) --cref -Map $(MAPFILE) -o $(FIRMWARE)

# The object file main.o depends on main.c.  main.c is compiled to make main.o
main.o: main.c
	$(CC) -c $(CCFLAGS) main.c -o main.o

init.o: init.c
	$(CC) -c $(CCFLAGS) init.c -o init.o


# if someone types in 'make clean' then remove all object files and executables
# associated wit this project
clean: 
	rm $(OBJS) 
	rm $(FIRMWARE) 
