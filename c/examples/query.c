#include <stdio.h>
#include <string.h>

#include <taiwan_lottery/query.h>

#include "../src/game_args.h"

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s local <game> period <PERIOD> [output_dir]\n", program);
    fprintf(stderr, "  %s local <game> month <YYYY-MM> [output_dir]\n", program);
    fprintf(stderr, "  %s local <game> month-range <YYYY-MM> <YYYY-MM> [output_dir]\n", program);
    fprintf(stderr, "  %s remote <game> period <PERIOD>\n", program);
    fprintf(stderr, "  %s remote <game> month <YYYY-MM>\n", program);
    fprintf(stderr, "  %s remote <game> month-range <YYYY-MM> <YYYY-MM>\n", program);
    fprintf(stderr, "  game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638 | bingo-bingo\n");
}


static void print_numbers(const int32_t *values, size_t len) {
    size_t i;
    printf("[");
    for (i = 0; i < len; ++i) {
        if (i > 0) {
            printf(", ");
        }
        printf("%d", values[i]);
    }
    printf("]");
}

int main(int argc, char **argv) {
    const char *program = argc > 0 ? argv[0] : "query";
    const char *default_output_dir = "data";

    if (argc < 5) {
        print_usage(program);
        return 2;
    }

    const char *source = argv[1];
    int game = taiwan_lottery_parse_game_arg(argv[2]);
    const char *query_mode = argv[3];
    int is_month_range = strcmp(query_mode, "month-range") == 0;
    int value_count = is_month_range ? 2 : 1;
    int status;
    taiwan_lottery_history_draw_page *page = NULL;

    if (game < 0) {
        fprintf(stderr, "Invalid game: %s\n", argv[2]);
        print_usage(program);
        return 2;
    }

    if (strcmp(query_mode, "period") != 0 && strcmp(query_mode, "month") != 0 && !is_month_range) {
        fprintf(stderr, "Query mode must be period, month, or month-range\n");
        print_usage(program);
        return 2;
    }

    if (strcmp(source, "local") == 0) {
        if (argc < 4 + value_count + 1) {
            fprintf(stderr, "Missing query value(s) for mode: %s\n", query_mode);
            print_usage(program);
            return 2;
        }
        const char *output_dir = argc > 4 + value_count ? argv[4 + value_count] : default_output_dir;
        const char *period = strcmp(query_mode, "period") == 0 ? argv[4] : NULL;
        const char *month = !period ? argv[4] : NULL;
        const char *end_month = is_month_range ? argv[5] : month;

        status = query_history_draw(output_dir, game, period, month, end_month, &page);
    } else if (strcmp(source, "remote") == 0) {
        if (argc < 4 + value_count) {
            fprintf(stderr, "Missing query value(s) for mode: %s\n", query_mode);
            print_usage(program);
            return 2;
        }

        const char *period = strcmp(query_mode, "period") == 0 ? argv[4] : NULL;
        const char *month = !period ? argv[4] : NULL;
        const char *end_month = is_month_range ? argv[5] : month;

        status = query_history_draw_from_taiwan_lottery(game, period, month, end_month, &page);
    } else {
        print_usage(program);
        return 2;
    }

    if (status != TAIWAN_LOTTERY_OK) {
        fprintf(stderr, "Query failed (status=%d)\n", status);
        return 1;
    }

    printf("total_size=%zu\n", page->total_size);
    for (size_t i = 0; i < page->item_count; ++i) {
        const taiwan_lottery_history_draw_item *item = &page->items[i];
        printf("period=%s\n", item->period != NULL ? item->period : "");
        printf("date=%s\n", item->date != NULL ? item->date : "");

        printf("numbers=");
        print_numbers(item->numbers.base.numbers, item->numbers.base.numbers_len);
        printf("\n");

        if (item->numbers.sorted_numbers != NULL && item->numbers.sorted_numbers_len > 0) {
            printf("numbers_sorted=");
            print_numbers(item->numbers.sorted_numbers, item->numbers.sorted_numbers_len);
            printf("\n\n");
        } else {
            printf("numbers_sorted=<not available>\n\n");
        }
    }

    free_history_draw_page(page);
    return 0;
}
