#include <numeric>
#include <sstream>

namespace gqlite::string
{
  /**
   * @internal
   * @return the result of joining the element of a vector with delimiter
   */
  inline std::string join(const std::vector<std::string>& _v, const std::string& _delimiter)
  {
    return std::accumulate(
      std::next(_v.begin()), 
      _v.end(), 
      _v[0], 
      [_delimiter](std::string a, std::string b) {
          return a + _delimiter + b;
      }
    );
  }
  /**
   * @internal
   * @return the result of spliting \p s with delimiter \p delim
   */
  inline std::vector<std::string> split (const std::string &s, char delim)
  {
    std::vector<std::string> result;
    std::stringstream ss (s);
    std::string item;

    while (std::getline (ss, item, delim))
    {
      if(not item.empty()) result.push_back (item);
    }

    return result;
  }
}
