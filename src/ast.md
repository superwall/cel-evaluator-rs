## AST example

For convenience, here is a JSON example of an AST that can be evaluated by the library.

```json
{
  "type": "And",
  "value": [
    {
      "type": "Relation",
      "value": [
        {
          "type": "Arithmetic",
          "value": [
            {
              "type": "Atom",
              "value": {
                "type": "Int",
                "value": 5
              }
            },
            {
              "type": "Add"
            },
            {
              "type": "Atom",
              "value": {
                "type": "Int",
                "value": 3
              }
            }
          ]
        },
        {
          "type": "GreaterThan"
        },
        {
          "type": "Atom",
          "value": {
            "type": "Int",
            "value": 7
          }
        }
      ]
    },
    {
      "type": "Relation",
      "value": [
        {
          "type": "FunctionCall",
          "value": [
            {
              "type": "Member",
              "value": [
                {
                  "type": "Ident",
                  "value": "name"
                },
                {
                  "type": "Attribute",
                  "value": "length"
                }
              ]
            },
            null,
            []
          ]
        },
        {
          "type": "In"
        },
        {
          "type": "List",
          "value": [
            {
              "type": "Atom",
              "value": {
                "type": "Int",
                "value": 5
              }
            },
            {
              "type": "Atom",
              "value": {
                "type": "Int",
                "value": 10
              }
            },
            {
              "type": "Atom",
              "value": {
                "type": "Int",
                "value": 15
              }
            }
          ]
        }
      ]
    }
  ]
}
```