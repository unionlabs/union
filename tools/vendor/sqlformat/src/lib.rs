//! This crate is a port of https://github.com/kufii/sql-formatter-plus
//! written in Rust. It is intended to be usable as a pure-Rust library
//! for formatting SQL queries.

#![type_length_limit = "99999999"]
#![forbid(unsafe_code)]
// Maintains semver compatibility for older Rust versions
#![allow(clippy::manual_strip)]

mod formatter;
mod indentation;
mod inline_block;
mod params;
mod tokenizer;

/// Formats whitespace in a SQL string to make it easier to read.
/// Optionally replaces parameter placeholders with `params`.
pub fn format(query: &str, params: &QueryParams, options: FormatOptions) -> String {
    let tokens = tokenizer::tokenize(query);
    formatter::format(&tokens, params, options)
}

/// Options for controlling how the library formats SQL
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Controls the type and length of indentation to use
    ///
    /// Default: 2 spaces
    pub indent: Indent,
    /// When set, changes reserved keywords to ALL CAPS
    ///
    /// Default: false
    pub uppercase: bool,
    /// Controls the number of line breaks after a query
    ///
    /// Default: 1
    pub lines_between_queries: u8,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            indent: Indent::Spaces(2),
            uppercase: false,
            lines_between_queries: 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Indent {
    Spaces(u8),
    Tabs,
}

#[derive(Debug, Clone)]
pub enum QueryParams {
    Named(Vec<(String, String)>),
    Indexed(Vec<String>),
    None,
}

impl Default for QueryParams {
    fn default() -> Self {
        QueryParams::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn it_uses_given_indent_config_for_indentation() {
        let input = "SELECT count(*),Column1 FROM Table1;";
        let mut options = FormatOptions::default();
        options.indent = Indent::Spaces(4);
        let expected = indoc!(
            "
            SELECT
                count(*),
                Column1
            FROM
                Table1;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_set_schema_queries() {
        let input = "SET SCHEMA schema1; SET CURRENT SCHEMA schema2;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SET SCHEMA
              schema1;
            SET CURRENT SCHEMA
              schema2;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_select_query() {
        let input = "SELECT count(*),Column1 FROM Table1;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              count(*),
              Column1
            FROM
              Table1;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_complex_select() {
        let input =
            "SELECT DISTINCT name, ROUND(age/7) field1, 18 + 20 AS field2, 'some string' FROM foo;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              DISTINCT name,
              ROUND(age / 7) field1,
              18 + 20 AS field2,
              'some string'
            FROM
              foo;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_with_complex_where() {
        let input = indoc!(
            "
            SELECT * FROM foo WHERE Column1 = 'testing'
            AND ( (Column2 = Column3 OR Column4 >= NOW()) );
      "
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *
            FROM
              foo
            WHERE
              Column1 = 'testing'
              AND (
                (
                  Column2 = Column3
                  OR Column4 >= NOW()
                )
              );"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_with_top_level_reserved_words() {
        let input = indoc!(
            "
            SELECT * FROM foo WHERE name = 'John' GROUP BY some_column
            HAVING column > 10 ORDER BY other_column LIMIT 5;
      "
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *
            FROM
              foo
            WHERE
              name = 'John'
            GROUP BY
              some_column
            HAVING
              column > 10
            ORDER BY
              other_column
            LIMIT
              5;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_limit_with_two_comma_separated_values_on_single_line() {
        let input = "LIMIT 5, 10;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            LIMIT
              5, 10;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_limit_of_single_value_followed_by_another_select_using_commas() {
        let input = "LIMIT 5; SELECT foo, bar;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            LIMIT
              5;
            SELECT
              foo,
              bar;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_limit_of_single_value_and_offset() {
        let input = "LIMIT 5 OFFSET 8;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            LIMIT
              5 OFFSET 8;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_limit_in_lowercase() {
        let input = "limit 5, 10;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            limit
              5, 10;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_preserves_case_of_keywords() {
        let input = "select distinct * frOM foo left join bar WHERe a > 1 and b = 3";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            select
              distinct *
            frOM
              foo
              left join bar
            WHERe
              a > 1
              and b = 3"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_query_with_select_query_inside_it() {
        let input = "SELECT *, SUM(*) AS sum FROM (SELECT * FROM Posts LIMIT 30) WHERE a > b";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *,
              SUM(*) AS sum
            FROM
              (
                SELECT
                  *
                FROM
                  Posts
                LIMIT
                  30
              )
            WHERE
              a > b"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_query_with_inner_join() {
        let input = indoc!(
            "
            SELECT customer_id.from, COUNT(order_id) AS total FROM customers
            INNER JOIN orders ON customers.customer_id = orders.customer_id;"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              customer_id.from,
              COUNT(order_id) AS total
            FROM
              customers
              INNER JOIN orders ON customers.customer_id = orders.customer_id;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_query_with_different_comments() {
        let input = indoc!(
            "
            SELECT
            /*
             * This is a block comment
             */
            * FROM
            -- This is another comment
            MyTable # One final comment
            WHERE 1 = 2;"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              /*
               * This is a block comment
               */
              *
            FROM
              -- This is another comment
              MyTable # One final comment
            WHERE
              1 = 2;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_maintains_block_comment_indentation() {
        let input = indoc!(
            "
            SELECT
              /*
               * This is a block comment
               */
              *
            FROM
              MyTable
            WHERE
              1 = 2;"
        );
        let options = FormatOptions::default();

        assert_eq!(format(input, &QueryParams::None, options), input);
    }

    #[test]
    fn it_formats_simple_insert_query() {
        let input = "INSERT INTO Customers (ID, MoneyBalance, Address, City) VALUES (12,-123.4, 'Skagen 2111','Stv');";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            INSERT INTO
              Customers (ID, MoneyBalance, Address, City)
            VALUES
              (12, -123.4, 'Skagen 2111', 'Stv');"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_keeps_short_parenthesized_list_with_nested_parenthesis_on_single_line() {
        let input = "SELECT (a + b * (c - NOW()));";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              (a + b * (c - NOW()));"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_breaks_long_parenthesized_lists_to_multiple_lines() {
        let input = indoc!(
            "
            INSERT INTO some_table (id_product, id_shop, id_currency, id_country, id_registration) (
            SELECT IF(dq.id_discounter_shopping = 2, dq.value, dq.value / 100),
            IF (dq.id_discounter_shopping = 2, 'amount', 'percentage') FROM foo);"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            INSERT INTO
              some_table (
                id_product,
                id_shop,
                id_currency,
                id_country,
                id_registration
              ) (
                SELECT
                  IF(
                    dq.id_discounter_shopping = 2,
                    dq.value,
                    dq.value / 100
                  ),
                  IF (
                    dq.id_discounter_shopping = 2,
                    'amount',
                    'percentage'
                  )
                FROM
                  foo
              );"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_update_query() {
        let input = "UPDATE Customers SET ContactName='Alfred Schmidt', City='Hamburg' WHERE CustomerName='Alfreds Futterkiste';";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            UPDATE
              Customers
            SET
              ContactName = 'Alfred Schmidt',
              City = 'Hamburg'
            WHERE
              CustomerName = 'Alfreds Futterkiste';"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_delete_query() {
        let input = "DELETE FROM Customers WHERE CustomerName='Alfred' AND Phone=5002132;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            DELETE FROM
              Customers
            WHERE
              CustomerName = 'Alfred'
              AND Phone = 5002132;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_drop_query() {
        let input = "DROP TABLE IF EXISTS admin_role;";
        let options = FormatOptions::default();

        assert_eq!(format(input, &QueryParams::None, options), input);
    }

    #[test]
    fn it_formats_incomplete_query() {
        let input = "SELECT count(";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              count("
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_query_that_ends_with_open_comment() {
        let input = indoc!(
            "
            SELECT count(*)
            /*Comment"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              count(*)
              /*Comment"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_update_query_with_as_part() {
        let input = "UPDATE customers SET total_orders = order_summary.total  FROM ( SELECT * FROM bank) AS order_summary";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            UPDATE
              customers
            SET
              total_orders = order_summary.total
            FROM
              (
                SELECT
                  *
                FROM
                  bank
              ) AS order_summary"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_top_level_and_newline_multi_word_reserved_words_with_inconsistent_spacing() {
        let input = "SELECT * FROM foo LEFT \t OUTER  \n JOIN bar ORDER \n BY blah";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *
            FROM
              foo
              LEFT OUTER JOIN bar
            ORDER BY
              blah"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_long_double_parenthesized_queries_to_multiple_lines() {
        let input = "((foo = '0123456789-0123456789-0123456789-0123456789'))";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            (
              (
                foo = '0123456789-0123456789-0123456789-0123456789'
              )
            )"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_short_double_parenthesizes_queries_to_one_line() {
        let input = "((foo = 'bar'))";
        let options = FormatOptions::default();

        assert_eq!(format(input, &QueryParams::None, options), input);
    }

    #[test]
    fn it_formats_single_char_operators() {
        let inputs = [
            "foo = bar",
            "foo < bar",
            "foo > bar",
            "foo + bar",
            "foo - bar",
            "foo * bar",
            "foo / bar",
            "foo % bar",
        ];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_formats_multi_char_operators() {
        let inputs = [
            "foo != bar",
            "foo <> bar",
            "foo == bar",
            "foo || bar",
            "foo <= bar",
            "foo >= bar",
            "foo !< bar",
            "foo !> bar",
        ];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_formats_logical_operators() {
        let inputs = [
            "foo ALL bar",
            "foo = ANY (1, 2, 3)",
            "EXISTS bar",
            "foo IN (1, 2, 3)",
            "foo LIKE 'hello%'",
            "foo IS NULL",
            "UNIQUE foo",
        ];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_formats_and_or_operators() {
        let strings = [
            ("foo BETWEEN bar AND baz", "foo BETWEEN bar\nAND baz"),
            ("foo AND bar", "foo\nAND bar"),
            ("foo OR bar", "foo\nOR bar"),
        ];
        let options = FormatOptions::default();
        for (input, output) in &strings {
            assert_eq!(&format(input, &QueryParams::None, options), output);
        }
    }

    #[test]
    fn it_recognizes_strings() {
        let inputs = ["\"foo JOIN bar\"", "'foo JOIN bar'", "`foo JOIN bar`"];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_recognizes_escaped_strings() {
        let inputs = [
            r#""foo \" JOIN bar""#,
            r#"'foo \' JOIN bar'"#,
            r#"`foo `` JOIN bar`"#,
            r#"'foo '' JOIN bar'"#,
            r#"'two households"'"#,
            r#"'two households'''"#,
        ];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_formats_postgres_specific_operators() {
        let strings = [
            ("column::int", "column :: int"),
            ("v->2", "v -> 2"),
            ("v->>2", "v ->> 2"),
            ("foo ~~ 'hello'", "foo ~~ 'hello'"),
            ("foo !~ 'hello'", "foo !~ 'hello'"),
            ("foo ~* 'hello'", "foo ~* 'hello'"),
            ("foo ~~* 'hello'", "foo ~~* 'hello'"),
            ("foo !~~ 'hello'", "foo !~~ 'hello'"),
            ("foo !~* 'hello'", "foo !~* 'hello'"),
            ("foo !~~* 'hello'", "foo !~~* 'hello'"),
        ];
        let options = FormatOptions::default();
        for (input, output) in &strings {
            assert_eq!(&format(input, &QueryParams::None, options), output);
        }
    }

    #[test]
    fn it_keeps_separation_between_multiple_statements() {
        let strings = [
            ("foo;bar;", "foo;\nbar;"),
            ("foo\n;bar;", "foo;\nbar;"),
            ("foo\n\n\n;bar;\n\n", "foo;\nbar;"),
        ];
        let options = FormatOptions::default();
        for (input, output) in &strings {
            assert_eq!(&format(input, &QueryParams::None, options), output);
        }

        let input = indoc!(
            "
            SELECT count(*),Column1 FROM Table1;
            SELECT count(*),Column1 FROM Table2;"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              count(*),
              Column1
            FROM
              Table1;
            SELECT
              count(*),
              Column1
            FROM
              Table2;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_unicode_correctly() {
        let input = "SELECT test, тест FROM table;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              test,
              тест
            FROM
              table;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_converts_keywords_to_uppercase_when_option_passed_in() {
        let input = "select distinct * frOM foo left join bar WHERe cola > 1 and colb = 3";
        let options = FormatOptions {
            uppercase: true,
            ..FormatOptions::default()
        };
        let expected = indoc!(
            "
            SELECT
              DISTINCT *
            FROM
              foo
              LEFT JOIN bar
            WHERE
              cola > 1
              AND colb = 3"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_line_breaks_between_queries_with_config() {
        let input = "SELECT * FROM foo; SELECT * FROM bar;";
        let options = FormatOptions {
            lines_between_queries: 2,
            ..FormatOptions::default()
        };
        let expected = indoc!(
            "
            SELECT
              *
            FROM
              foo;

            SELECT
              *
            FROM
              bar;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_correctly_indents_create_statement_after_select() {
        let input = indoc!(
            "
            SELECT * FROM test;
            CREATE TABLE TEST(id NUMBER NOT NULL, col1 VARCHAR2(20), col2 VARCHAR2(20));
        "
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *
            FROM
              test;
            CREATE TABLE TEST(
              id NUMBER NOT NULL,
              col1 VARCHAR2(20),
              col2 VARCHAR2(20)
            );"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_short_create_table() {
        let input = "CREATE TABLE items (a INT PRIMARY KEY, b TEXT);";
        let options = FormatOptions::default();

        assert_eq!(format(input, &QueryParams::None, options), input);
    }

    #[test]
    fn it_formats_long_create_table() {
        let input =
            "CREATE TABLE items (a INT PRIMARY KEY, b TEXT, c INT NOT NULL, d INT NOT NULL);";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            CREATE TABLE items (
              a INT PRIMARY KEY,
              b TEXT,
              c INT NOT NULL,
              d INT NOT NULL
            );"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_insert_without_into() {
        let input =
      "INSERT Customers (ID, MoneyBalance, Address, City) VALUES (12,-123.4, 'Skagen 2111','Stv');";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            INSERT
              Customers (ID, MoneyBalance, Address, City)
            VALUES
              (12, -123.4, 'Skagen 2111', 'Stv');"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_alter_table_modify_query() {
        let input = "ALTER TABLE supplier MODIFY supplier_name char(100) NOT NULL;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            ALTER TABLE
              supplier
            MODIFY
              supplier_name char(100) NOT NULL;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_alter_table_alter_column_query() {
        let input = "ALTER TABLE supplier ALTER COLUMN supplier_name VARCHAR(100) NOT NULL;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            ALTER TABLE
              supplier
            ALTER COLUMN
              supplier_name VARCHAR(100) NOT NULL;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_bracketed_strings() {
        let inputs = ["[foo JOIN bar]", "[foo ]] JOIN bar]"];
        let options = FormatOptions::default();
        for input in &inputs {
            assert_eq!(&format(input, &QueryParams::None, options), input);
        }
    }

    #[test]
    fn it_recognizes_at_variables() {
        let input =
            "SELECT @variable, @a1_2.3$, @'var name', @\"var name\", @`var name`, @[var name];";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              @variable,
              @a1_2.3$,
              @'var name',
              @\"var name\",
              @`var name`,
              @[var name];"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_at_variables_with_param_values() {
        let input =
            "SELECT @variable, @a1_2.3$, @'var name', @\"var name\", @`var name`, @[var name], @'var\\name';";
        let params = vec![
            ("variable".to_string(), "\"variable value\"".to_string()),
            ("a1_2.3$".to_string(), "'weird value'".to_string()),
            ("var name".to_string(), "'var value'".to_string()),
            ("var\\name".to_string(), "'var\\ value'".to_string()),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              \"variable value\",
              'weird value',
              'var value',
              'var value',
              'var value',
              'var value',
              'var\\ value';"
        );

        assert_eq!(
            format(input, &QueryParams::Named(params), options),
            expected
        );
    }

    #[test]
    fn it_recognizes_colon_variables() {
        let input =
            "SELECT :variable, :a1_2.3$, :'var name', :\"var name\", :`var name`, :[var name];";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              :variable,
              :a1_2.3$,
              :'var name',
              :\"var name\",
              :`var name`,
              :[var name];"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_colon_variables_with_param_values() {
        let input = indoc!(
            "
            SELECT :variable, :a1_2.3$, :'var name', :\"var name\", :`var name`,
            :[var name], :'escaped \\'var\\'', :\"^*& weird \\\" var   \";
            "
        );
        let params = vec![
            ("variable".to_string(), "\"variable value\"".to_string()),
            ("a1_2.3$".to_string(), "'weird value'".to_string()),
            ("var name".to_string(), "'var value'".to_string()),
            ("escaped 'var'".to_string(), "'weirder value'".to_string()),
            (
                "^*& weird \" var   ".to_string(),
                "'super weird value'".to_string(),
            ),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              \"variable value\",
              'weird value',
              'var value',
              'var value',
              'var value',
              'var value',
              'weirder value',
              'super weird value';"
        );

        assert_eq!(
            format(input, &QueryParams::Named(params), options),
            expected
        );
    }

    #[test]
    fn it_recognizes_question_numbered_placeholders() {
        let input = "SELECT ?1, ?25, ?;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              ?1,
              ?25,
              ?;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_question_numbered_placeholders_with_param_values() {
        let input = "SELECT ?1, ?2, ?0;";
        let params = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              second,
              third,
              first;"
        );

        assert_eq!(
            format(input, &QueryParams::Indexed(params), options),
            expected
        );

        format("?62666666121266666612", &QueryParams::None, options);
    }

    #[test]
    fn it_recognizes_question_indexed_placeholders_with_param_values() {
        let input = "SELECT ?, ?, ?;";
        let params = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              first,
              second,
              third;"
        );

        assert_eq!(
            format(input, &QueryParams::Indexed(params), options),
            expected
        );
    }

    #[test]
    fn it_recognizes_dollar_sign_numbered_placeholders() {
        let input = "SELECT $1, $2;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              $1,
              $2;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_dollar_sign_numbered_placeholders_with_param_values() {
        let input = "SELECT $2, $3, $1;";
        let params = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              second,
              third,
              first;"
        );

        assert_eq!(
            format(input, &QueryParams::Indexed(params), options),
            expected
        );
    }

    #[test]
    fn it_formats_query_with_go_batch_separator() {
        let input = "SELECT 1 GO SELECT 2";
        let params = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              1
            GO
            SELECT
              2"
        );

        assert_eq!(
            format(input, &QueryParams::Indexed(params), options),
            expected
        );
    }

    #[test]
    fn it_formats_select_query_with_cross_join() {
        let input = "SELECT a, b FROM t CROSS JOIN t2 on t.id = t2.id_t";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a,
              b
            FROM
              t
              CROSS JOIN t2 on t.id = t2.id_t"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_query_with_cross_apply() {
        let input = "SELECT a, b FROM t CROSS APPLY fn(t.id)";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a,
              b
            FROM
              t
              CROSS APPLY fn(t.id)"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_select() {
        let input = "SELECT N, M FROM t";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              N,
              M
            FROM
              t"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_simple_select_with_national_characters_mssql() {
        let input = "SELECT N'value'";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              N'value'"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_select_query_with_outer_apply() {
        let input = "SELECT a, b FROM t OUTER APPLY fn(t.id)";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a,
              b
            FROM
              t
              OUTER APPLY fn(t.id)"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_fetch_first_like_limit() {
        let input = "SELECT * FETCH FIRST 2 ROWS ONLY;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              *
            FETCH FIRST
              2 ROWS ONLY;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_case_when_with_a_blank_expression() {
        let input = "CASE WHEN option = 'foo' THEN 1 WHEN option = 'bar' THEN 2 WHEN option = 'baz' THEN 3 ELSE 4 END;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            CASE
              WHEN option = 'foo' THEN 1
              WHEN option = 'bar' THEN 2
              WHEN option = 'baz' THEN 3
              ELSE 4
            END;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_case_when_inside_select() {
        let input =
            "SELECT foo, bar, CASE baz WHEN 'one' THEN 1 WHEN 'two' THEN 2 ELSE 3 END FROM table";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              foo,
              bar,
              CASE
                baz
                WHEN 'one' THEN 1
                WHEN 'two' THEN 2
                ELSE 3
              END
            FROM
              table"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_case_when_with_an_expression() {
        let input = "CASE toString(getNumber()) WHEN 'one' THEN 1 WHEN 'two' THEN 2 WHEN 'three' THEN 3 ELSE 4 END;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            CASE
              toString(getNumber())
              WHEN 'one' THEN 1
              WHEN 'two' THEN 2
              WHEN 'three' THEN 3
              ELSE 4
            END;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_recognizes_lowercase_case_end() {
        let input = "case when option = 'foo' then 1 else 2 end;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            case
              when option = 'foo' then 1
              else 2
            end;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_ignores_words_case_and_end_inside_other_strings() {
        let input = "SELECT CASEDATE, ENDDATE FROM table1;";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              CASEDATE,
              ENDDATE
            FROM
              table1;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_tricky_line_comments() {
        let input = "SELECT a#comment, here\nFROM b--comment";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a #comment, here
            FROM
              b --comment"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_line_comments_followed_by_semicolon() {
        let input = indoc!(
            "
            SELECT a FROM b
            --comment
            ;"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a
            FROM
              b --comment
            ;"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_line_comments_followed_by_comma() {
        let input = indoc!(
            "
            SELECT a --comment
            , b"
        );
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a --comment
            ,
              b"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_line_comments_followed_by_close_paren() {
        let input = "SELECT ( a --comment\n )";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              (
                a --comment
              )"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_line_comments_followed_by_open_paren() {
        let input = "SELECT a --comment\n()";
        let options = FormatOptions::default();
        let expected = indoc!(
            "
            SELECT
              a --comment
              ()"
        );

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }

    #[test]
    fn it_formats_lonely_semicolon() {
        let input = ";";
        let options = FormatOptions::default();

        assert_eq!(format(input, &QueryParams::None, options), input);
    }

    #[test]
    fn it_formats_multibyte_chars() {
        let input = "\nSELECT 'главная'";
        let options = FormatOptions::default();
        let expected = "SELECT\n  'главная'";

        assert_eq!(format(input, &QueryParams::None, options), expected);
    }
}
