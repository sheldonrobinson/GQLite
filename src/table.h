#include <gqlite.h>

#include <span>
#include <string>
#include <unordered_map>
#include <vector>

#include "gqlite_p.h"
#include "errors.h"

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
    class row_view : public std::span<const _T_>
    {
      using std::span<const _T_>::span;
    public:
      operator std::vector<_T_>() const
      {
        return std::vector<_T_>(this->begin(), this->end());
      }
      template<template<typename T> class C>
      bool operator==(const C<_T_>& _v) const
      {
        return this->size() == _v.size() and std::equal(this->begin(), this->end(), _v.begin(), _v.end());
      }
    };
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
    bool has_column(const std::string& _column) const;
    std::size_t get_column_index(const std::string& _column) const;
    /**
     * @return the name of the columns
     */
    std::vector<std::string> get_columns_names() const;
    std::size_t get_columns_count() const;
    std::size_t get_rows_count() const;
    /**
     * @return the row at index \p _j
     */
    row_view get_row(std::size_t _j) const;
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
    errors::check_condition(_row.size() == get_columns_count(), "Invalid row size, got {}, expected {}", _row.size(), get_columns_count());
    m_values.insert(m_values.end(), _row.begin(), _row.end());
    ++m_rows;
  }
  template<typename _T_>
  bool table<_T_>::has_column(const std::string& _column) const
  {
    return m_labels.find(_column) != m_labels.end();
  }
  template<typename _T_>
  std::size_t table<_T_>::get_column_index(const std::string& _column) const
  {
    auto it = m_labels.find(_column);
    if(it != m_labels.end()) return it->second;
    throw exception("Unknown column {}");
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
  inline table<_T_>::row_view table<_T_>::get_row(std::size_t _j) const
  {
    if(_j >= m_rows)
    {
      throw exception("Invalid index {}, size is {}", _j, m_rows);
    }
    return row_view(m_values.begin() + _j * m_columns, m_values.begin() + (_j+1) * m_columns);
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
