/*****************************************************************************
* | File      	:   DEV_Config.c
* | Author      :   Waveshare team
* | Function    :   Hardware underlying interface
* | Info        :
*----------------
* |	This version:   V1.0
* | Date        :   2020-12-21
* | Info        :
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documnetation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of theex Software, and to permit persons to  whom the Software is
# furished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS OR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
#
******************************************************************************/
#include "DEV_Config.h"
#include <fcntl.h>
/* for fallback to spidev-based hardware SPI if bcm2835 fails */
#ifdef RPI
#include "dev_hardware_SPI.h"
#include "RPI_sysfs_gpio.h"
#endif

/**
 * GPIO
**/
int DEV_RST_PIN;
int DEV_CS_PIN;
int DEV_DRDY_PIN;

/* runtime selector: use bcm2835 lib when available, otherwise use sysfs/dev interface */
static int use_bcm2835 = 0;

/* GPIO offset for sysfs: 0 for Pi 4 and earlier, 571 for Pi 5 */
static int gpio_sysfs_offset = 0;

static int detect_pi5_gpio_offset(void)
{
	/* On Raspberry Pi 5, the RP1 GPIO chip has base 571 */
	FILE *fp = fopen("/sys/class/gpio/gpiochip571/label", "r");
	if (fp) {
		char label[64];
		if (fgets(label, sizeof(label), fp)) {
			fclose(fp);
			if (strstr(label, "pinctrl-rp1") != NULL) {
				return 571;
			}
		} else {
			fclose(fp);
		}
	}
	return 0;
}
/**
 * GPIO read and write
**/
void DEV_Digital_Write(UWORD Pin, UBYTE Value)
{
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if (use_bcm2835) {
		bcm2835_gpio_write(Pin, Value);
	} else {
		SYSFS_GPIO_Write(Pin, Value);
	}
#elif USE_WIRINGPI_LIB
	digitalWrite(Pin, Value);
#elif USE_DEV_LIB
	SYSFS_GPIO_Write(Pin, Value);
#endif
#endif

#ifdef JETSON
#ifdef USE_DEV_LIB
	SYSFS_GPIO_Write(Pin, Value);
#elif USE_HARDWARE_LIB
	Debug("not support");
#endif
#endif
}

UBYTE DEV_Digital_Read(UWORD Pin)
{
	UBYTE Read_value = 0;
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if (use_bcm2835) {
		Read_value = bcm2835_gpio_lev(Pin);
	} else {
		Read_value = SYSFS_GPIO_Read(Pin);
	}
#elif USE_WIRINGPI_LIB
	Read_value = digitalRead(Pin);
#elif USE_DEV_LIB
	Read_value = SYSFS_GPIO_Read(Pin);
#endif
#endif

#ifdef JETSON
#ifdef USE_DEV_LIB
	Read_value = SYSFS_GPIO_Read(Pin);
#elif USE_HARDWARE_LIB
	Debug("not support");
#endif
#endif
	return Read_value;
}

/**
 * SPI
**/
UBYTE DEV_SPI_WriteByte(uint8_t Value)
{
	UBYTE temp = 0;
	// printf("write %x \r\n", Value);
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if (use_bcm2835) {
		temp = bcm2835_spi_transfer(Value);
	} else {
		temp = DEV_HARDWARE_SPI_TransferByte(Value);
	}
#elif USE_WIRINGPI_LIB
	wiringPiSPIDataRW(0, &Value, 1);
	temp = Value;
#elif USE_DEV_LIB
	temp = DEV_HARDWARE_SPI_TransferByte(Value);
#endif
#endif

#ifdef JETSON
#ifdef USE_DEV_LIB
	temp = SYSFS_software_spi_transfer(Value);
#elif USE_HARDWARE_LIB
	Debug("not support");
#endif
#endif
	// printf("Read %x \r\n", temp);
	return temp;
}

UBYTE DEV_SPI_ReadByte(void)
{
	return DEV_SPI_WriteByte(0x00);
}

/**
 * GPIO Mode
**/
void DEV_GPIO_Mode(UWORD Pin, UWORD Mode)
{
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if (use_bcm2835) {
		if(Mode == 0 || Mode == BCM2835_GPIO_FSEL_INPT) {
			bcm2835_gpio_fsel(Pin, BCM2835_GPIO_FSEL_INPT);
		} else {
			bcm2835_gpio_fsel(Pin, BCM2835_GPIO_FSEL_OUTP);
		}
	} else {
		SYSFS_GPIO_Export(Pin);
		if(Mode == 0 || Mode == SYSFS_GPIO_IN) {
			SYSFS_GPIO_Direction(Pin, SYSFS_GPIO_IN);
		} else {
			SYSFS_GPIO_Direction(Pin, SYSFS_GPIO_OUT);
		}
	}
#elif USE_WIRINGPI_LIB
	if(Mode == 0 || Mode == INPUT) {
		pinMode(Pin, INPUT);
		pullUpDnControl(Pin, PUD_UP);
	} else {
		pinMode(Pin, OUTPUT);
	}
#elif USE_DEV_LIB
	SYSFS_GPIO_Export(Pin);
	if(Mode == 0 || Mode == SYSFS_GPIO_IN) {
		SYSFS_GPIO_Direction(Pin, SYSFS_GPIO_IN);
	} else {
		SYSFS_GPIO_Direction(Pin, SYSFS_GPIO_OUT);
	}
#endif
#endif

#ifdef JETSON
#ifdef USE_DEV_LIB
	SYSFS_GPIO_Export(Pin);
	SYSFS_GPIO_Direction(Pin, Mode);
#elif USE_HARDWARE_LIB
	Debug("not support");
#endif
#endif
}

/**
 * delay x ms
**/
void DEV_Delay_ms(UDOUBLE xms)
{
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if (use_bcm2835) {
		bcm2835_delay(xms);
	} else {
		UDOUBLE i;
		for(i=0; i < xms; i++) {
			usleep(1000);
		}
	}
#elif USE_WIRINGPI_LIB
	delay(xms);
#elif USE_DEV_LIB
	UDOUBLE i;
	for(i=0; i < xms; i++) {
		usleep(1000);
	}
#endif
#endif

#ifdef JETSON
	UDOUBLE i;
	for(i=0; i < xms; i++) {
		usleep(1000);
	}
#endif
}

static int DEV_Equipment_Testing(void)
{
	int i;
	int fd;
	char value_str[20];
	fd = open("/etc/issue", O_RDONLY);
    printf("Current environment: ");
	while(1) {
		if (fd < 0) {
			Debug( "Read failed Pin\n");
			return -1;
		}
		for(i=0;; i++) {
			if (read(fd, &value_str[i], 1) < 0) {
				Debug( "failed to read value!\n");
				return -1;
			}
			if(value_str[i] ==32) {
				printf("\r\n");
				/* null-terminate the string we read (stop at first space) */
				value_str[i] = '\0';
				break;
			}
			printf("%c",value_str[i]);
		}
		break;
	}
#ifdef RPI
	if(i<1) {
		printf("Unrecognizable\r\n");
	} else {
		/* Accept common Debian-based images (Raspbian, Debian, Raspberry Pi OS) */
		if (strncmp(value_str, "Raspbian", 8) == 0 || strncmp(value_str, "Debian", 6) == 0 || strncmp(value_str, "Raspberry", 9) == 0) {
			/* ok */
		} else {
			printf("Warning: OS is '%s' (not Raspbian). Continuing anyway.\r\n", value_str);
		}
	}
#endif
#ifdef JETSON
	if(i<5) {
		Debug("Unrecognizable\r\n");
	} else {
		char JETSON_System[10]= {"Ubuntu"};
		for(i=0; i<6; i++) {
			if(JETSON_System[i]!= value_str[i] ) {
				printf("Please make RPI !!!!!!!!!!\r\n");
				return -1;
			}
		}
	}
#endif
	return 0;
}

void DEV_GPIO_Init(void)
{
#ifdef RPI
	DEV_RST_PIN     = 18;
	DEV_CS_PIN      = 22;
	DEV_DRDY_PIN    = 17;
#elif JETSON
	DEV_RST_PIN     = GPIO18;
	DEV_CS_PIN      = GPIO22;
	DEV_DRDY_PIN    = GPIO17;
#endif

	DEV_GPIO_Mode(DEV_RST_PIN, 1);
	DEV_GPIO_Mode(DEV_CS_PIN, 1);
	
	DEV_GPIO_Mode(DEV_DRDY_PIN, 0);
	
	DEV_Digital_Write(DEV_CS_PIN, 1);
}

/******************************************************************************
function:	Module Initialize, the library and initialize the pins, SPI protocol
parameter:
Info:
******************************************************************************/
UBYTE DEV_Module_Init(void)
{
    printf("/***********************************/ \r\n");
	if(DEV_Equipment_Testing() < 0) {
		return 1;
	}
#ifdef RPI
#ifdef USE_BCM2835_LIB
	if(!bcm2835_init()) {
		printf("bcm2835_init failed; attempting fallback to /dev/spidev...\r\n");
		use_bcm2835 = 0;
		/* Detect Pi 5 GPIO offset (571 for Pi 5, 0 for earlier) */
		gpio_sysfs_offset = detect_pi5_gpio_offset();
		if (gpio_sysfs_offset > 0) {
			printf("Raspberry Pi 5 detected: using GPIO offset %d\r\n", gpio_sysfs_offset);
		}
		/* Try using spidev-based hardware SPI as a fallback */
		/* Initialize GPIOs using sysfs with Pi 5 offset */
		DEV_RST_PIN     = 18 + gpio_sysfs_offset;
		DEV_CS_PIN      = 22 + gpio_sysfs_offset;
		DEV_DRDY_PIN    = 17 + gpio_sysfs_offset;
		SYSFS_GPIO_Export(DEV_RST_PIN);
		SYSFS_GPIO_Export(DEV_CS_PIN);
		SYSFS_GPIO_Export(DEV_DRDY_PIN);
		SYSFS_GPIO_Direction(DEV_RST_PIN, SYSFS_GPIO_OUT);
		SYSFS_GPIO_Direction(DEV_CS_PIN, SYSFS_GPIO_OUT);
		SYSFS_GPIO_Direction(DEV_DRDY_PIN, SYSFS_GPIO_IN);
		DEV_HARDWARE_SPI_begin("/dev/spidev0.0");
		DEV_HARDWARE_SPI_setSpeed(1000000);
		DEV_HARDWARE_SPI_Mode(SPI_MODE_1);
	} else {
		printf("bcm2835 init success !!! \r\n");
		use_bcm2835 = 1;

		/* GPIO Config */
		DEV_GPIO_Init();

		bcm2835_spi_begin();                                         //Start spi interface, set spi pin for the reuse function
		bcm2835_spi_setBitOrder(BCM2835_SPI_BIT_ORDER_MSBFIRST);     //High first transmission
		bcm2835_spi_setDataMode(BCM2835_SPI_MODE1);                  //spi mode 1, '0, 1'
		bcm2835_spi_setClockDivider(BCM2835_SPI_CLOCK_DIVIDER_32);  //Frequency
	}
	/* Runtime diagnostics */
	if (use_bcm2835) {
		printf("Runtime backend: bcm2835 (direct /dev/gpiomem)\r\n");
		printf("GPIO pins: RST=%d CS=%d DRDY=%d\r\n", DEV_RST_PIN, DEV_CS_PIN, DEV_DRDY_PIN);
		printf("SPI: bcm2835 mode, clock divider set by bcm2835 library\r\n");
	} else {
		printf("Runtime backend: spidev/sysfs fallback\r\n");
		printf("Using SPI device: /dev/spidev0.0 at 1MHz (configured)\r\n");
		printf("GPIO via sysfs: RST=%d CS=%d DRDY=%d\r\n", DEV_RST_PIN, DEV_CS_PIN, DEV_DRDY_PIN);
	}
#elif USE_WIRINGPI_LIB
	// if(wiringPiSetup() < 0) {//use wiringpi Pin number table
	if(wiringPiSetupGpio() < 0) { //use BCM2835 Pin number table
		printf("set wiringPi lib failed	!!! \r\n");
		return 1;
	} else {
		printf("set wiringPi lib success !!! \r\n");
	}

	// GPIO Config
	DEV_GPIO_Init();
	// wiringPiSPISetup(0,10000000);
	wiringPiSPISetupMode(0, 1000000, 1);
#elif USE_DEV_LIB
	printf("Write and read /dev/spidev0.0 \r\n");
	DEV_GPIO_Init();
	DEV_HARDWARE_SPI_begin("/dev/spidev0.0");
    DEV_HARDWARE_SPI_setSpeed(1000000);
	DEV_HARDWARE_SPI_Mode(SPI_MODE_1);
#endif


#elif JETSON
#ifdef USE_DEV_LIB
	DEV_GPIO_Init();
	printf("Software spi\r\n");
	SYSFS_software_spi_begin();
	SYSFS_software_spi_setBitOrder(SOFTWARE_SPI_MSBFIRST);
	SYSFS_software_spi_setDataMode(SOFTWARE_SPI_Mode1);
	SYSFS_software_spi_setClockDivider(SOFTWARE_SPI_CLOCK_DIV16);
#elif USE_HARDWARE_LIB
	printf("Write and read /dev/spidev0.0 \r\n");
	DEV_GPIO_Init();
	DEV_HARDWARE_SPI_begin("/dev/spidev0.0");
#endif

#endif
    printf("/***********************************/ \r\n");
	return 0;
}

/******************************************************************************
function:	Module exits, closes SPI and BCM2835 library
parameter:
Info:
******************************************************************************/
void DEV_Module_Exit(void)
{
#ifdef RPI
#ifdef USE_BCM2835_LIB
	DEV_Digital_Write(DEV_RST_PIN, LOW);
	DEV_Digital_Write(DEV_CS_PIN, LOW);
	if (use_bcm2835) {
		bcm2835_spi_end();
		bcm2835_close();
	} else {
		DEV_HARDWARE_SPI_end();
	}
#elif USE_WIRINGPI_LIB
	DEV_Digital_Write(DEV_RST_PIN, 0);
	DEV_Digital_Write(DEV_CS_PIN, 0);
#elif USE_DEV_LIB
	DEV_HARDWARE_SPI_end();
	DEV_Digital_Write(DEV_RST_PIN, 0);
	DEV_Digital_Write(DEV_CS_PIN, 0);
#endif

#elif JETSON
#ifdef USE_DEV_LIB
	SYSFS_GPIO_Unexport(DEV_RST_PIN);
	SYSFS_GPIO_Unexport(DEV_CS_PIN);
	SYSFS_GPIO_Unexport(DEV_DRDY_PIN);

#elif USE_HARDWARE_LIB
	Debug("not support");
#endif
#endif
}
