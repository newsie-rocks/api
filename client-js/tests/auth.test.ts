import { Client, User } from '../src/index';
import { faker } from '@faker-js/faker';

let client: Client;
let test_user: User;
let test_token: string;
beforeEach(async () => {
  client = new Client('http://localhost:3000');

  let name = faker.person.fullName();
  let email = faker.internet.email();
  let password = 'enter';
  let [user, token] = await client.signup({
    name,
    email,
    password,
  });
  test_user = user;
  test_token = token;
  client.token = token;
});

afterEach(async () => {
  await client.deleteMe();
});

test('login', async () => {
  let [user, _token] = await client.login(test_user.email, 'enter');
  expect(user.name).toBe(test_user.name);
  expect(user.email).toBe(test_user.email);
});

test('update user', async () => {
  let name = faker.person.fullName();
  let user = await client.updateMe({ name });
  expect(user.name).toBe(name);
});
