#include <gqlite.h>

#include <string>
#include <unordered_map>
#include <vector>

namespace gqlite
{
  /**
   * @internal
   * 
   * This class allow to store data as a table, during the execution of the query.
   */
  template<typename _T_>
  class table
  {
  public:
    table();
    ~table();
    /**
     * Add a single column. Adding column to a table with data is a costly operation.
     */
    void add_column(const std::string& _column, const _T_& _val = _T_());
    /**
     * Add a set of columns. Adding column to a table with data is a costly operation.
     */
    void add_columns(const std::vector<std::string>& _columns, const _T_& _val = _T_());
    /**
     * Add a row to the table.
     */
    void add_row(const std::vector<_T_>& _row);
    /**
     * @return the name of the columns
     */
    std::vector<std::string> get_columns_names() const;
    std::size_t get_columns_count() const;
    std::size_t get_rows_count() const;
    /**
     * @return the row at index \p _j
     */
    std::vector<_T_> get_row(std::size_t _j);
    /**
     * @return the value at column \p _i and row \p _j
     */
    const _T_& get_value(std::size_t _i, std::size_t _j) const;
    /**
     * @return the value at column \p _column and row \p _j
     */
    const _T_& get_value(const std::string& _column, std::size_t _j) const;
  private:
    void add_columns(std::size_t _count, const _T_& _val);
    std::size_t m_columns = 0, m_rows = 0;
    std::unordered_map<std::string, int> m_labels;
    std::vector<_T_> m_values;
  };

  template<typename _T_>
  inline table<_T_>::table() {}
  template<typename _T_>
  inline table<_T_>::~table() {}
  template<typename _T_>
  inline void table<_T_>::add_column(const std::string& _column, const _T_& _val)
  {
    add_columns(1, _val);
    m_labels[_column] = m_labels.size();
  }
  template<typename _T_>
  inline void table<_T_>::add_columns(const std::vector<std::string>& _columns, const _T_& _val)
  {
    add_columns(_columns.size(), _val);
    for(const std::string& s : _columns)
    {
      m_labels[s] = m_labels.size();
    }
  }
  template<typename _T_>
  inline void table<_T_>::add_row(const std::vector<_T_>& _row)
  {
    m_values.insert(m_values.end(), _row.begin(), _row.end());
    ++m_rows;
  }
  template<typename _T_>
  inline std::vector<std::string> table<_T_>::get_columns_names() const
  {
    std::vector<std::string> col;
    col.resize(m_columns);
    for(auto const& [k, v] : m_labels)
    {
      col[v] = k;
    }
    return col;
  }
  template<typename _T_>
  inline std::size_t table<_T_>::get_columns_count() const
  {
    return m_columns;
  }
  template<typename _T_>
  inline std::size_t table<_T_>::get_rows_count() const
  {
    return m_rows;
  }
  template<typename _T_>
  inline std::vector<_T_> table<_T_>::get_row(std::size_t _j)
  {
    std::vector<_T_> row;
    row.reserve(m_columns);
    row.insert(row.end(), m_values.begin() + _j * m_columns, m_values.begin() + (_j+1) * m_columns);
    return row;
  }
  template<typename _T_>
  inline const _T_& table<_T_>::get_value(std::size_t _i, std::size_t _j) const
  {
    return m_values[_i + _j * m_columns];
  }
  template<typename _T_>
  inline const _T_& table<_T_>::get_value(const std::string& _column, std::size_t _j) const
  {
    auto it = m_labels.find(_column);
    if(it == m_labels.end()) throw gqlite::exception("Unknown column {}", _column);
    return get_value(it->second, _j);
  }
  template<typename _T_>
  inline void table<_T_>::add_columns(std::size_t _count, const _T_& _val)
  {
    std::size_t new_columns = m_columns + _count;
    if(m_rows > 0)
    {
      std::vector<_T_> new_values;
      new_values.reserve(new_columns * m_rows);
      auto new_value_bi = std::back_inserter(new_values);
      for(std::size_t j = 0; j < m_rows; ++j)
      {
        std::copy(m_values.begin() + m_columns * j, m_values.begin()  + m_columns * (j+1), new_value_bi);
        std::fill_n(new_value_bi, _count, _val);
      }
      m_values = new_values;
    }
    m_columns = new_columns;
  }
}
