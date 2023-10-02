#include <istream>

namespace gqlite::oc
{

class token;
class lexer
{
  public:
    struct mode
    {
      static constexpr int normal = 0x0;
      static constexpr int disable_keywords = 0x1;
    };
  public:
    lexer(std::istream* sstream);
    ~lexer();
  public:
    token next_token(int _flags = mode::normal);
    void unget(const token& _token);
  protected:
    bool ignore_comment(token& _token, int _lastChar);
    /**
     * @return the next char and increment the column counter.
     */
    int get_next_char();
    /**
     * @return the next char that is not a separator (space, tab, return...)
     */
    int get_next_non_separator_char();
    /**
     * Cancel the reading of the previous char.
     */
    void unget();
    bool eof() const;
    int line() const;
    int column() const;
    /**
     * Get an identifier (or keyword) in the current flow of character.
     */
    std::string get_identifier(int lastChar);
    token get_string(int lastChar);
  private:
    struct data;
    data* const d;
};

} // namespace gqlite::oc
