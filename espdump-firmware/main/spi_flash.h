// spi_flash.h
#pragma once

#include <stdint.h>

// Init
void spi_flash_init(void);

// ID
void spi_read_jedec_id(uint8_t *buf);

// Read fast
// Reads 'len' bytes starting at 'addr' into 'buf;
void spi_flash_read(uint32_t addr, uint8_t *buf, uint32_t len);

// Write control 
void spi_flash_write_enable(void);
void spi_flash_wait_busy(void);

// Erase 
// Erases a 4KB sector containing 'addr'
void spi_flash_sector_erase(uint32_t addr);

// Program
// Programs up to one 256 byte page
void spi_flash_page_program(uint32_t addr, u_int8_t *buf, uint32_t len);