// Wait for DOM loaded before setup
document.addEventListener("DOMContentLoaded", () => {
  console.log("DOMContentLoaded fired");
  const page = window.location.pathname;
  console.log("Current page:", page);

  if (page.endsWith("login.html") || page === "/") {
    if (!document.getElementById("loginForm")?.onsubmit) {
      setupLogin();
    }
  }

  if (page.endsWith("register.html")) {
    if (!document.getElementById("registerForm")?.onsubmit) {
      setupRegister();
    }
  }

  if (page.endsWith("admin_dashboard.html")) {
    setupLogout();
  }

  if (page.endsWith("admin_books.html")) {
    loadBooks(false); // admin view
    setupAddBook();
    setupLogout();
  }

  if (page.endsWith("lender_dashboard.html")) {
    setupLogout();
  }

  if (page.endsWith("browse_books.html")) {
    loadBooks(true); // browse mode (borrow buttons)
    setupSearch();
    setupLogout();
  }

  if (page.endsWith("my_books.html")) {
    loadBorrowedBooks();
    setupLogout();
  }

  if (page.endsWith("return_books.html")) {
    loadBorrowedBooks();
    setupLogout();
  }

  if (page.endsWith("due_books.html")) {
    loadOverdueBooks();
    setupLogout();
  }

  if (page.endsWith("admin_users.html")) {
    loadUsers();
    setupLogout();
  }

  if (page.endsWith("admin_borrowed.html")) {
    loadAdminBorrowedBooks();
    setupLogout();
  }

  if (page.endsWith("admin_overdue.html")) {
    loadAdminOverdueBooks();
    setupLogout();
  }

  if (page.endsWith("edit_book.html")) {
    setupEditBook();
    setupLogout();
  }
});

// Utility: Get cookie value by name
function getCookie(name) {
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop().split(";").shift();
  return null;
}

// ======== AUTH & USER MANAGEMENT ======== //

function setupLogin() {
  const form = document.getElementById("loginForm");
  if (!form) {
    console.error("Login form not found");
    return;
  }
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    const data = {
      username: form.username.value.trim(),
      password: form.password.value.trim(),
      role: form.role.value.trim(),
    };
    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const json = await res.json();
      if (res.ok) {
        document.cookie = `username=${json.username}; path=/`;
        document.cookie = `user_id=${json.user_id}; path=/`;
        if (json.role === "admin") {
          window.location.href = "/admin_dashboard.html";
        } else {
          window.location.href = "/lender_dashboard.html";
        }
      } else {
        document.getElementById("error").textContent = json.error || "Login failed";
      }
    } catch (err) {
      document.getElementById("error").textContent = "Error connecting to server";
    }
  });
}

function setupRegister() {
  const form = document.getElementById("registerForm");
  if (!form) {
    console.error("Register form not found");
    return;
  }
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    const data = {
      username: form.username.value.trim(),
      email: form.email.value.trim(),
      password: form.password.value.trim(),
      role: form.role ? form.role.value.trim() : "lender",
    };
    try {
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const json = await res.json();
      if (res.ok) {
        window.location.href = "/login.html";
      } else {
        document.getElementById("error").textContent = json.error || "Registration failed";
      }
    } catch (err) {
      document.getElementById("error").textContent = "Error connecting to server";
    }
  });
}

function setupLogout() {
  const logoutBtn = document.getElementById("logoutBtn");
  if (!logoutBtn) return;
  logoutBtn.addEventListener("click", () => {
    document.cookie = "username=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
    document.cookie = "user_id=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
    window.location.href = "/login.html";
  });
}

// ======== BOOK MANAGEMENT ======== //

function loadBooks(browse = false) {
  const username = getCookie("username");
  const role = username === "admin" ? "admin" : "lender";
  const endpoint = browse ? "/api/admin/books" : "/api/admin";

  fetch(endpoint, { headers: { "Content-Type": "application/json" } })
    .then((res) => res.json())
    .then((data) => {
      const bookList = document.getElementById("bookList") || document.getElementById("bookAdminList");
      if (!bookList) return;
      bookList.innerHTML = "";
      data.forEach((book) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${book.title} by ${book.author} (ISBN: ${book.isbn}) - ${book.copies_available} available</span>
          ${
            browse
              ? `<button onclick="borrowBook(${book.id})">Borrow</button>`
              : role === "admin"
              ? `<button onclick="editBook(${book.id})">Edit</button>
                 <button onclick="deleteBook(${book.id})">Delete</button>`
              : ""
          }
        `;
        bookList.appendChild(li);
      });
    })
    .catch(() => {
      const errEl = document.getElementById("error");
      if (errEl) errEl.textContent = "Error loading books";
    });
}

// ======== FIXED: ADD BOOK FORM SETUP ======== //

function setupAddBook() {
  const form = document.getElementById("addBookForm");
  if (!form) {
    console.error("Add book form not found");
    return;
  }

  form.addEventListener("submit", async (e) => {
    e.preventDefault();

    // Validate form inputs
    const title = form.title.value.trim();
    const author = form.author.value.trim();
    const isbn = form.isbn.value.trim();
    const publicationYear = parseInt(form.publication_year.value);
    const genre = form.genre.value.trim();
    const copiesAvailable = parseInt(form.copies_available.value);

    if (!title || !author || !isbn || isNaN(publicationYear) || !genre || isNaN(copiesAvailable)) {
      document.getElementById("error").textContent = "Please fill all fields with valid values";
      return;
    }

    const data = {
      title,
      author,
      isbn,
      publication_year: publicationYear,
      genre,
      copies_available: copiesAvailable,
      status: "available"
    };

    try {
      const res = await fetch("/api/book", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data)
      });

      if (!res.ok) {
        const errorData = await res.json();
        throw new Error(errorData.error || "Failed to add book");
      }

      // Success handling
      form.reset();
      document.getElementById("error").textContent = "";
      
      // If on admin books page, refresh the list
      if (window.location.pathname.endsWith("admin_books.html")) {
        loadBooks(false);
      } else {
        // Redirect to admin books page if not already there
        window.location.href = "admin_books.html";
      }
      
    } catch (error) {
      console.error("Error adding book:", error);
      document.getElementById("error").textContent = error.message || "Error adding book";
    }
  });
}




function borrowBook(bookId) {
  const userId = getCookie("user_id");
  if (!userId) {
    alert("Please login to borrow a book.");
    return;
  }
  fetch("/api/borrow", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ book_id: bookId, user_id: parseInt(userId) }),
  })
    .then((res) => res.json())
    .then((json) => {
      if (json.status === "borrowed") {
        alert("Book borrowed successfully");
        loadBooks(true);
      } else {
        alert(json.error || "Failed to borrow book");
      }
    })
    .catch(() => {
      alert("Error connecting to server");
    });
}

function editBook(bookId) {
  window.location.href = `/edit_book.html?id=${bookId}`;
}

function deleteBook(bookId) {
  fetch(`/api/book`, {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ id: bookId }),
  })
    .then((res) => res.json())
    .then((json) => {
      if (json.status === "book deleted") {
        alert("Book deleted successfully");
        loadBooks(false);
      } else {
        alert(json.error || "Failed to delete book");
      }
    })
    .catch(() => {
      alert("Error connecting to server");
    });
}

function setupEditBook() {
  const urlParams = new URLSearchParams(window.location.search);
  const bookId = urlParams.get("id");
  if (!bookId) {
    alert("Invalid book ID");
    return;
  }
  fetch(`/api/book?id=${bookId}`)
    .then((res) => res.json())
    .then((book) => {
      const form = document.getElementById("editBookForm");
      if (!form) return;
      form.title.value = book.title || "";
      form.author.value = book.author || "";
      form.isbn.value = book.isbn || "";
      form.publication_year.value = book.publication_year || "";
      form.genre.value = book.genre || "";
      form.copies_available.value = book.copies_available || "";
      form.status.value = book.status || "available";

      form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const data = {
          id: parseInt(bookId),
          title: form.title.value.trim(),
          author: form.author.value.trim(),
          isbn: form.isbn.value.trim(),
          publication_year: parseInt(form.publication_year.value),
          genre: form.genre.value.trim(),
          copies_available: parseInt(form.copies_available.value),
          status: form.status.value.trim(),
        };
        try {
          const res = await fetch("/api/book", {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data),
          });
          const json = await res.json();
          if (res.ok) {
            alert("Book updated successfully");
            window.location.href = "/admin_books.html";
          } else {
            alert(json.error || "Failed to edit book");
          }
        } catch {
          alert("Error connecting to server");
        }
      });
    })
    .catch(() => {
      alert("Error loading book details");
    });
}

function setupSearch() {
  const searchInput = document.getElementById("searchBox") || document.getElementById("searchInput");
  if (!searchInput) return;
  searchInput.addEventListener("input", () => {
    const query = searchInput.value.trim();
    fetch(`/api/admin/books?search=${encodeURIComponent(query)}`)
      .then((res) => res.json())
      .then((data) => {
        const bookList = document.getElementById("bookList") || document.getElementById("bookAdminList");
        if (!bookList) return;
        bookList.innerHTML = "";
        data.forEach((book) => {
          const li = document.createElement("li");
          li.innerHTML = `
            <span>${book.title} by ${book.author} (ISBN: ${book.isbn}) - ${book.copies_available} available</span>
            <button onclick="borrowBook(${book.id})">Borrow</button>
          `;
          bookList.appendChild(li);
        });
      })
      .catch(() => {
        alert("Error searching books");
      });
  });
}

// ======== BORROW/RETURN BOOKS ======== //

function loadBorrowedBooks() {
  const userId = getCookie("user_id");
  if (!userId) {
    alert("Please login first.");
    return;
  }
  fetch(`/api/borrow?user_id=${userId}`)
    .then((res) => res.json())
    .then((data) => {
      const bookList = document.getElementById("borrowedBooks") || document.getElementById("returnBookList") || document.getElementById("bookList");
      if (!bookList) return;
      bookList.innerHTML = "";
      if (data.length === 0) {
        bookList.innerHTML = "<p>No books currently borrowed.</p>";
        return;
      }
      data.forEach((record) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${record.title} by ${record.author} - Due: ${record.due_date}</span>
          <button onclick="returnBook(${record.id})">Return</button>
        `;
        bookList.appendChild(li);
      });
    })
    .catch(() => {
      alert("Error loading borrowed books");
    });
}

function returnBook(recordId) {
  fetch(`/api/borrow`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ record_id: recordId }),
  })
    .then((res) => res.json())
    .then((json) => {
      if (json.status === "returned") {
        alert("Book returned successfully");
        loadBorrowedBooks();
      } else {
        alert(json.error || "Failed to return book");
      }
    })
    .catch(() => {
      alert("Error connecting to server");
    });
}

function loadOverdueBooks() {
  const userId = getCookie("user_id");
  if (!userId) {
    alert("Please login first.");
    return;
  }
  fetch(`/api/borrow/overdue?user_id=${userId}`)
    .then((res) => res.json())
    .then((data) => {
      const bookList = document.getElementById("dueBooks") || document.getElementById("bookList");
      if (!bookList) return;
      bookList.innerHTML = "";
      if (data.length === 0) {
        bookList.innerHTML = "<p>No overdue books.</p>";
        return;
      }
      data.forEach((record) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${record.title} by ${record.author} - Due: ${record.due_date}</span>
          <button onclick="returnBook(${record.id})">Return</button>
        `;
        bookList.appendChild(li);
      });
    })
    .catch(() => {
      alert("Error loading overdue books");
    });
}

// ======== ADMIN USERS MANAGEMENT ======== //

function loadUsers() {
  fetch("/api/admin/users")
    .then((res) => res.json())
    .then((data) => {
      const userList = document.getElementById("userList");
      if (!userList) return;
      userList.innerHTML = "";
      if (data.length === 0) {
        userList.innerHTML = "<p>No registered users found.</p>";
        return;
      }
      data.forEach((user) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${user.username} (${user.email}) - Role: ${user.role}</span>
          <button onclick="deleteUser(${user.id})">Delete</button>
        `;
        userList.appendChild(li);
      });
    })
    .catch(() => {
      alert("Error loading users");
    });
}

function deleteUser(userId) {
  fetch(`/api/admin/users?id=${userId}`, {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
  })
    .then((res) => res.json())
    .then((json) => {
      if (json.status === "deleted") {
        alert("User deleted successfully");
        loadUsers();
      } else {
        alert(json.error || "Failed to delete user");
      }
    })
    .catch(() => {
      alert("Error connecting to server");
    });
}

// ======== ADMIN BORROWED & OVERDUE BOOKS ======== //

function loadAdminBorrowedBooks() {
  fetch("/api/admin/borrowed")
    .then((res) => res.json())
    .then((data) => {
      const bookList = document.getElementById("adminBorrowedList") || document.getElementById("bookList");
      if (!bookList) return;
      bookList.innerHTML = "";
      if (data.length === 0) {
        bookList.innerHTML = "<p>No books currently borrowed.</p>";
        return;
      }
      data.forEach((record) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${record.title} by ${record.author} - Borrowed by ${record.username} - Due: ${record.due_date}</span>
        `;
        bookList.appendChild(li);
      });
    })
    .catch(() => {
      alert("Error loading borrowed books");
    });
}

function loadAdminOverdueBooks() {
  fetch("/api/admin/overdue")
    .then((res) => res.json())
    .then((data) => {
      const bookList = document.getElementById("overdueList") || document.getElementById("bookList");
      if (!bookList) return;
      bookList.innerHTML = "";
      if (data.length === 0) {
        bookList.innerHTML = "<p>No overdue books.</p>";
        return;
      }
      data.forEach((record) => {
        const li = document.createElement("li");
        li.innerHTML = `
          <span>${record.title} by ${record.author} - Borrowed by ${record.username} - Due: ${record.due_date}</span>
        `;
        bookList.appendChild(li);
      });
    })
    .catch(() => {
      alert("Error loading overdue books");
    });
}
