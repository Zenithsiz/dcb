#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <string.h>

/// Checks if a buffer is null
int is_null(const unsigned char *buf, size_t size)
{
	for (size_t n = 0; n < size; n++)
	{
		if (buf[n] != '\0')
		{
			return 0;
		}
	}

	return 1;
}

int main(int argc, char **argv)
{
	// If we didn't get an argument, return err
	if (argc != 2)
	{
		fprintf(stderr, "Usage:\n\t./get_tables <file-name>");
		return EXIT_FAILURE;
	}

	// Open input and output files
	FILE *in = fopen(argv[1], "rb");
	FILE *out = fopen("all-tables.txt", "w");

	// Reading buffer
	unsigned char buffer[0x100];

	// Previous buffer
	unsigned char prev_buffer[0x100] = {255};

	// Bytes read
	size_t bytes_read = 0;

	// Input size
	fseek(in, 0, SEEK_END);
	size_t input_size = ftell(in);
	fseek(in, 0, SEEK_SET);

	// Read until we reach EOF
	while (bytes_read < input_size)
	{
		// Get current address
		size_t address = bytes_read;

		// Read 0x100 bytes
		memcpy(prev_buffer, buffer, 0x100 * sizeof(char));
		fread(buffer, sizeof(char), 0x100, in);
		bytes_read += 0x100;

		// If the header isn't normal ascii, discard
		if (iscntrl(buffer[0]) || buffer[0] > 0x7f ||
			iscntrl(buffer[1]) || buffer[1] > 0x7f ||
			iscntrl(buffer[2]) || buffer[2] > 0x7f ||
			iscntrl(buffer[3]) || buffer[3] > 0x7f)
		{
			continue;
		}

		// If this buffer is all NULL, discard
		if (is_null(buffer, 0x100))
		{
			continue;
		}

		// If the last buffer wasn't all NULL, discard
		if (!is_null(prev_buffer, 0x100))
		{
			continue;
		}

		// Else write it to output after formatting
		fprintf(out, "%04x: %c%c%c%c\n", address, buffer[0], buffer[1], buffer[2], buffer[3]);
	}

	// Return success
	return EXIT_SUCCESS;
}
