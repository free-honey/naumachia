use aiken/interval.{Interval, IntervalBound, NegativeInfinity, PositiveInfinity, Finite}

pub fn contains_interval(outer: Interval<Int>, inner: Interval<Int>) -> Bool {
    outer_is_at_least_as_low_as_inner(outer.lower_bound, inner.lower_bound) && outer_is_at_least_as_high_as_inner(outer.upper_bound, inner.upper_bound)
}

fn outer_is_at_least_as_low_as_inner(outer: IntervalBound<Int>, inner: IntervalBound<Int>) -> Bool {
    when outer.bound_type is {
        NegativeInfinity -> True
        Finite(out_bound) -> {
            when inner.bound_type is {
                NegativeInfinity -> False
                Finite(in_bound) -> {
                    if out_bound < in_bound {
                        True
                    } else if out_bound > in_bound {
                        False
                    } else {
                        // out_bound == in_bound
                        if !outer.is_inclusive && inner.is_inclusive{
                            False
                        } else {
                            True
                        }
                    }
                }
                PositiveInfinity -> True
            }
        }
        PositiveInfinity -> False 
    }
}

fn outer_is_at_least_as_high_as_inner(outer: IntervalBound<Int>, inner: IntervalBound<Int>) -> Bool {
    when outer.bound_type is {
        NegativeInfinity -> False
        Finite(out_bound) -> {
            when inner.bound_type is {
                NegativeInfinity -> True 
                Finite(in_bound) -> {
                    if out_bound > in_bound {
                        True
                    } else if out_bound < in_bound {
                        False
                    } else {
                        // out_bound == in_bound
                        if !outer.is_inclusive && inner.is_inclusive{
                            False
                        } else {
                            True
                        }
                    }
                }
                PositiveInfinity -> False 
            }
        }
        PositiveInfinity -> True
    }
}

pub fn contains(interval: Interval<Int>, val: Int) -> Bool {
    lower_bound_fits(interval.lower_bound, val) && upper_bound_fits(interval.upper_bound, val)
}

fn lower_bound_fits(bound: IntervalBound<Int>, val: Int) -> Bool {
    when bound.bound_type is {
        NegativeInfinity -> True 
        Finite(lower_bound) -> {
            if bound.is_inclusive {
               val >= lower_bound
            } else {
               val > lower_bound
            }
        }
        PositiveInfinity -> False
    }
}

fn upper_bound_fits(bound: IntervalBound<Int>, val: Int) -> Bool {
    when bound.bound_type is {
        NegativeInfinity -> False 
        Finite(lower_bound) -> {
            if bound.is_inclusive {
               val <= lower_bound
            } else {
               val < lower_bound
            }
        }
        PositiveInfinity -> True 
    }
}

pub fn infinite_range() -> Interval<Int> {
    let lower = IntervalBound { bound_type: NegativeInfinity, is_inclusive: True }
    let upper = IntervalBound { bound_type: PositiveInfinity, is_inclusive: True }
    Interval { lower_bound: lower, upper_bound: upper }
}

pub fn lower_bound_range(lower: Int, lower_incl: Bool) -> Interval<Int> {
    let lower_bound = IntervalBound { bound_type: Finite(lower), is_inclusive: lower_incl }
    let upper_bound = IntervalBound { bound_type: PositiveInfinity, is_inclusive: True }
    Interval { lower_bound: lower_bound, upper_bound: upper_bound }
}

pub fn upper_bound_range(upper: Int, upper_incl: Bool) -> Interval<Int> {
    let lower_bound = IntervalBound { bound_type: NegativeInfinity, is_inclusive: True }
    let upper_bound = IntervalBound { bound_type: Finite(upper), is_inclusive: upper_incl }
    Interval { lower_bound: lower_bound, upper_bound: upper_bound }
}

test contains_interval__infinity_can_fit_in_infinity() {
    let outer = infinite_range()
    let inner = infinite_range()
    contains_interval(outer, inner) == True
}

test contains_interval__infinity_cannot_fit_in_lower_bound() {
    let outer = lower_bound_range(1, True)
    let inner = infinite_range()
    contains_interval(outer, inner) == False
}

test contains_interval__outer_lower_higher_than_inner_false() {
    let outer = lower_bound_range(5, True)
    let inner = lower_bound_range(2, True)
    contains_interval(outer, inner) == False
}

test contains_interval__outer_lower_lower_than_inner_true() {
    let outer = lower_bound_range(2, True)
    let inner = lower_bound_range(5, True)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_lower_both_incl_true() {
    let outer = lower_bound_range(1, True)
    let inner = lower_bound_range(1, True)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_lower_just_outer_incl_true() {
    let outer = lower_bound_range(1, True)
    let inner = lower_bound_range(1, False)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_lower_just_outer_not_incl_false() {
    let outer = lower_bound_range(1, False)
    let inner = lower_bound_range(1, True)
    contains_interval(outer, inner) == False 
}

test contains_interval__matching_finite_lower_both_not_incl_true() {
    let outer = lower_bound_range(1, False)
    let inner = lower_bound_range(1, False)
    contains_interval(outer, inner) == True
}

test contains_interval__infinity_cannot_fit_in_upper_bound() {
    let outer = upper_bound_range(10, True)
    let inner = infinite_range()
    contains_interval(outer, inner) == False
}

test contains_interval__outer_upper_lower_than_inner_false() {
    let outer = upper_bound_range(8, True)
    let inner = upper_bound_range(10, True)
    contains_interval(outer, inner) == False
}

test contains_interval__outer_upper_higher_than_inner_true() {
    let outer = upper_bound_range(10, True)
    let inner = upper_bound_range(8, True)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_upper_both_incl_true() {
    let outer = upper_bound_range(10, True)
    let inner = upper_bound_range(10, True)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_upper_just_outer_incl_true() {
    let outer = upper_bound_range(10, True)
    let inner = upper_bound_range(10, False)
    contains_interval(outer, inner) == True
}

test contains_interval__matching_finite_upper_just_outer_not_incl_false() {
    let outer = upper_bound_range(1, False)
    let inner = upper_bound_range(1, True)
    contains_interval(outer, inner) == False 
}

test contains_interval__matching_finite_upper_both_not_incl_true() {
    let outer = lower_bound_range(1, False)
    let inner = lower_bound_range(1, False)
    contains_interval(outer, inner) == True
}

test contains__infinity_true() {
    let range = infinite_range()
    let val = 5
    contains(range, val) == True
}

test contains__lower_range_true() {
    let range = lower_bound_range(2, True)
    let val = 5
    contains(range, val) == True
}

test contains__below_lower_range_false() {
    let range = lower_bound_range(2, True)
    let val = 5
    contains(range, val) == True
}

test contains__lower_range_false() {
    let range = lower_bound_range(9, True)
    let val = 5
    contains(range, val) == False
}

test contains__upper_range_true() {
    let range = upper_bound_range(8, False)
    let val = 5
    contains(range, val) == True
}

test contains__upper_range_false() {
    let range = upper_bound_range(4, False)
    let val = 5
    contains(range, val) == False
}