import { createClient } from '@supabase/supabase-js';

const supabaseUrl = 'https://wwqpylbrcpriyaqugzsi.supabase.co';
const supabaseAnonKey = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Ind3cXB5bGJyY3ByaXlhcXVnenNpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjI0MzMyMjQsImV4cCI6MjAzODAwOTIyNH0.UQOmQ-wE63O32lyrLDO7ryowrM5LNA2UILHDA7hTH8E';

export const supabase = createClient(supabaseUrl, supabaseAnonKey);