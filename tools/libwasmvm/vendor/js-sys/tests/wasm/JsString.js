exports.new_string_object = () => new String("hi");

exports.get_replacer_function = function() {
	return function upperToHyphenLower(match, offset, string) {
		return (offset > 0 ? '-' : '') + match.toLowerCase();
	};
};
