#include <gba_base.h>
#include <gba_video.h>

void poke(int x, int y, u16 color);
void wait_line(u8 line);

int main(void) {
	wait_line(SCREEN_HEIGHT);

	SetMode(MODE_3 | BG2_ENABLE);

	int center = SCREEN_HEIGHT / 2;
	int line_width = 8;
	int line_y_min = center - (line_width / 2);
	int line_y_max = center + (line_width / 2);

	for (int y = 0; y < SCREEN_HEIGHT; y++) {
		u16 color = y >= line_y_min && y <= line_y_max ? RGB5(24, 10, 24) : RGB5(16, 28, 16);
		for (int x = 0; x < SCREEN_WIDTH; x++) {
			poke(x, y, color);
		}
	}

	wait_line(SCREEN_HEIGHT);
	*((vu32*)0x02000000) = 0xDEADBEEF;

	while (1);
	return 0;
}

void poke(int x, int y, u16 color) {
	MODE3_FB[y][x] = color;
}

void wait_line(u8 line) {
	while ((u8)REG_VCOUNT == line);
	while ((u8)REG_VCOUNT != line);
}