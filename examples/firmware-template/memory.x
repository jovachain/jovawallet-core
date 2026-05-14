/* Phase 7 firmware-template — generic Cortex-M memory map.
 * Sized to fit any STM32F4 / nRF52840 / SAMD51 dev board. Real firmware
 * substitutes the exact platform's memory.x.
 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
