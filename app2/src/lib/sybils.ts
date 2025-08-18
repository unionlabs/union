interface Person {
  name: string;
}

export function antiSybil(persons: Person[]): Person[] {
  return persons.filter(person => !person.name.includes("jessica"));
}