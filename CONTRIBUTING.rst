Commit guidelines
=================

The git-commit "format"::

    <type>:<scope>: <subject>

    <body>

The ``<type>`` SHOULD be present. The ``<scope>`` is OPTIONAL. The ``<body>`` is RECOMMENDED.

.. note::
    The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL
    NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED",
    "MAY", and "OPTIONAL" in this document are to be interpreted as
    described in BCP 14 [RFC2119] [RFC8174] when, and only when, they
    appear in all capitals, as shown here.

``<type>``
----------

- ``feature``
- ``fix``
- ``docs``
- ``tests``
- ``fmt`` - reformatting the code
- ``refactor``
- ``build`` - related to build & CI/CD systems
- ``meta`` - related to repository itself (e.g. .gitignore changes)
- ``bootstrap`` - in the early stage of repo, commits meant to get basic features and prepare repo for further development
