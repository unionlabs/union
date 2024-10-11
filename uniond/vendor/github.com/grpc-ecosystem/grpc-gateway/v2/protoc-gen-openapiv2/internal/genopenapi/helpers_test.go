package genopenapi

import (
	"reflect"
	"testing"
)

func Test_getUniqueFields(t *testing.T) {
	type args struct {
		schemaFieldsRequired []string
		fieldsRequired       []string
	}
	var tests = []struct {
		name string
		args args
		want []string
	}{
		{
			name: "test_1",
			args: args{
				schemaFieldsRequired: []string{"Field_1", "Field_2", "Field_3"},
				fieldsRequired:       []string{"Field_2"},
			},
			want: []string{"Field_1", "Field_3"},
		},
		{
			name: "test_2",
			args: args{
				schemaFieldsRequired: []string{"Field_1", "Field_2", "Field_3"},
				fieldsRequired:       []string{"Field_3"},
			},
			want: []string{"Field_1", "Field_2"},
		},
		{
			name: "test_3",
			args: args{
				schemaFieldsRequired: []string{"Field_1", "Field_2", "Field_3"},
				fieldsRequired:       []string{"Field_4"},
			},
			want: []string{"Field_1", "Field_2", "Field_3"},
		},
		{
			name: "test_4",
			args: args{
				schemaFieldsRequired: []string{"Field_1", "Field_2", "Field_3", "Field_4", "Field_5", "Field_6"},
				fieldsRequired:       []string{"Field_6", "Field_4", "Field_1"},
			},
			want: []string{"Field_2", "Field_3", "Field_5"},
		},
		{
			name: "test_5",
			args: args{
				schemaFieldsRequired: []string{"Field_1", "Field_2", "Field_3"},
				fieldsRequired:       []string{},
			},
			want: []string{"Field_1", "Field_2", "Field_3"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := getUniqueFields(tt.args.schemaFieldsRequired, tt.args.fieldsRequired); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("getUniqueFields() = %v, want %v", got, tt.want)
			}
		})
	}
}
