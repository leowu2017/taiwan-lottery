#include <assert.h>
#include <stddef.h>
#include <stdint.h>

#include <taiwan_lottery/numbers.h>

int main(void) {
    int32_t values[] = {1, 2, 3};
    int32_t sorted_values[] = {1, 2, 3};
    draw_numbers base = {values, 3};
    bonus_draw_numbers with_bonus = {base, 1, 4};
    sorted_draw_numbers with_sorted = {base, sorted_values, 3};

    assert(base.numbers == values);
    assert(base.numbers_len == 3);
    assert(with_bonus.base.numbers_len == 3);
    assert(with_bonus.has_bonus == 1);
    assert(with_bonus.bonus == 4);
    assert(with_sorted.base.numbers[2] == 3);
    assert(with_sorted.sorted_numbers_len == 3);
    assert(with_sorted.sorted_numbers[0] == 1);

    return 0;
}