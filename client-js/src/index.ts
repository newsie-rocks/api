/**
 * This module is the API client
 * @module newsie_client
 */

/**
 * API error
 */
export interface ApiError {
  error: {
    code: string;
    message: string;
    details: string;
  };
}

/**
 * New user
 */
export interface NewUser {
  name: string;
  email: string;
  password: string;
}

/**
 * User
 */
export interface User {
  id: string;
  name: string;
  email: string;
}

/**
 * User fields
 */
export interface UserFields {
  name?: string;
  email?: string;
  password?: string;
}

/**
 * API client
 */
export class Client {
  /**
   * API endpoint URL
   */
  url: string;
  /**
   * Authentication token
   */
  token: string | null = null;

  constructor(url: string) {
    this.url = url;
  }

  /**
   * Signups a new user
   */
  async signup(new_user: NewUser): Promise<[User, string]> {
    let res = await fetch(`${this.url}/auth/signup`, {
      method: 'POST',
      headers: {
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : undefined),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(new_user),
    });

    if (res.status < 300) {
      let data: { user: User; token: string } = await res.json();
      this.token = data.token;
      return [data.user, data.token];
    } else {
      let err: ApiError = await res.json();
      throw err;
    }
  }

  /**
   * Login a new user
   */
  async login(email: string, password: string): Promise<[User, string]> {
    let res = await fetch(`${this.url}/auth/login`, {
      method: 'POST',
      headers: {
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : undefined),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    });

    if (res.status < 300) {
      let data: { user: User; token: string } = await res.json();
      this.token = data.token;
      return [data.user, data.token];
    } else {
      let err: ApiError = await res.json();
      throw err;
    }
  }

  /**
   * Get the user info
   */
  async me(): Promise<User> {
    let res = await fetch(`${this.url}/auth/me`, {
      method: 'GET',
      headers: {
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : undefined),
      },
    });

    if (res.status < 300) {
      let data: { user: User } = await res.json();
      return data.user;
    } else {
      let err: ApiError = await res.json();
      throw err;
    }
  }

  /**
   * Update a user
   */
  async updateMe(fields: UserFields): Promise<User> {
    let res = await fetch(`${this.url}/auth/me`, {
      method: 'PATCH',
      headers: {
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : undefined),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(fields),
    });

    if (res.status < 300) {
      let data: { user: User } = await res.json();
      return data.user;
    } else {
      let err: ApiError = await res.json();
      throw err;
    }
  }

  /**
   * Delete a user
   */
  async deleteMe(): Promise<void> {
    let res = await fetch(`${this.url}/auth/me`, {
      method: 'DELETE',
      headers: {
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : undefined),
      },
    });

    if (res.status < 300) {
      this.token = null;
    } else {
      let err: ApiError = await res.json();
      throw err;
    }
  }
}
