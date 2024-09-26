/* useful reference: https://www.linuxselfhelp.com/gnu/ld/html_chapter/ld_toc.html  */
/* linker syntax reference: https://ftp.gnu.org/old-gnu/Manuals/ld-2.9.1/html_chapter/ld_3.html
/* Scaffolding for 'hello led' assignment. */
/* Course# 17638 Engineering Embedded Systems */
MEMORY
{
    flash : org = 0x08000000, len = 64k
    ram : org =   0x20000000, len = 12k
    ccmram : org = 0x10000000, len = 4k
}
  
/* No Modifications needed in SECTIONS. */
SECTIONS
{
	. = ORIGIN(flash);
	.text : 
	{
		*(.vectors); /* The interrupt vectors */
		*(.text);
	} >flash
	. = ORIGIN(ram);

    .data : 
	{
		INIT_DATA_VALUES = LOADADDR(.data);
		INIT_DATA_START = .;
		*(.data);
		INIT_DATA_END = .;
    } >ram AT>flash
	BSS_START = .;
	
	.bss : 
	{	  
		*(.bss);  
	} > ram
	BSS_END = .;
}
