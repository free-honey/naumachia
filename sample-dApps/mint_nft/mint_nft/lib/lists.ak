use context.{ScriptContext, Input}

pub fn list_contains_item(list: List(a), item: a) -> Bool {
    when list is {
        [item, ..tail] -> True
        [_, ..tail] -> list_contains_item(tail, item)
        [] -> False
    }
}