use context.{ScriptContext, Input}

pub fn list_contains_item(list: List<a>, item: a) -> Bool {
    when list is {
        [item, ..tail] -> True
        [_, ..tail] -> list_contains_item(tail, item)
        [] -> False
    }
}

pub fn map_on_list(list: List<a>, predicate: fn(a) -> b) -> List<b> {
    when list is {
        [item, ..tail] -> [predicate(item), ..map_on_list(tail, predicate)]
        [] -> []
    }
}

pub fn any_in_list(list: List<a>, predicate: fn(a) -> Bool) -> Bool {
    when list is {
        [item, ..tail] -> if predicate(item) {
                True
            } else {
                any_in_list(tail, predicate)
        }
        [] -> False
    }
}


test any_in_list_always_true() {
    let list = [1,2,3,4,5]
    let always_true = fn(_) { True }
    any_in_list(list, always_true)
}