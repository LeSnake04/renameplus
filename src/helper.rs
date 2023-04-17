pub fn into_none_if<T>(cond: bool, obj: T) -> Option<T> {
	if cond {
		None
	} else {
		Some(obj)
	}
}
